use crate::{
    EditableSettings, Settings,
    client::Client,
    login_provider::{LoginProvider, PendingLogin},
    metrics::PlayerMetrics,
    settings::{BanOperation, banlist::NormalizedIpAddr},
    sys::sentinel::TrackedStorages,
};
use authc::Uuid;
use common::{
    comp::{self, Admin, Player, Stats},
    event::{ClientDisconnectEvent, EventBus, MakeAdminEvent},
    recipe::{default_component_recipe_book, default_repair_recipe_book},
    resources::TimeOfDay,
    shared_server_config::ServerConstants,
    uid::Uid,
};
use common_base::prof_span;
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::{
    CharacterInfo, ClientRegister, DisconnectReason, PlayerInfo, PlayerListUpdate, RegisterError,
    ServerGeneral, ServerInit, WorldMapMsg, server::ServerDescription,
};
use hashbrown::{HashMap, hash_map};
use itertools::Either;
use rayon::prelude::*;
use specs::{
    Entities, Join, LendJoin, ParJoin, Read, ReadExpect, ReadStorage, SystemData, WriteExpect,
    WriteStorage, shred,
};
use tracing::{debug, info, trace, warn};

#[cfg(feature = "plugins")]
use common_state::plugin::PluginMgr;

#[derive(SystemData)]
pub struct ReadData<'a> {
    entities: Entities<'a>,
    stats: ReadStorage<'a, Stats>,
    uids: ReadStorage<'a, Uid>,
    client_disconnect_events: Read<'a, EventBus<ClientDisconnectEvent>>,
    make_admin_events: Read<'a, EventBus<MakeAdminEvent>>,
    login_provider: ReadExpect<'a, LoginProvider>,
    player_metrics: ReadExpect<'a, PlayerMetrics>,
    settings: ReadExpect<'a, Settings>,
    time_of_day: Read<'a, TimeOfDay>,
    material_stats: ReadExpect<'a, comp::item::MaterialStatManifest>,
    ability_map: ReadExpect<'a, comp::item::tool::AbilityMap>,
    recipe_book: ReadExpect<'a, common::recipe::RecipeBookManifest>,
    map: ReadExpect<'a, WorldMapMsg>,
    trackers: TrackedStorages<'a>,
    #[cfg(feature = "plugins")]
    plugin_mgr: Read<'a, PluginMgr>,
    data_dir: ReadExpect<'a, crate::DataDir>,
}

/// This system will handle new messages from clients
#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        ReadData<'a>,
        WriteStorage<'a, Client>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, PendingLogin>,
        WriteExpect<'a, EditableSettings>,
    );

    const NAME: &'static str = "msg::register";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (read_data, mut clients, mut players, mut pending_logins, mut editable_settings): Self::SystemData,
    ) {
        let mut make_admin_emitter = read_data.make_admin_events.emitter();
        // Player list to send new players, and lookup from UUID to entity (so we don't
        // have to do a linear scan over all entities on each login to see if
        // it's a duplicate).
        //
        // NOTE: For this to work as desired, we must maintain the invariant that there
        // is just one player per UUID!
        let (player_list, old_players_by_uuid): (HashMap<_, _>, HashMap<_, _>) = (
            &read_data.entities,
            &read_data.uids,
            clients.maybe(),
            &players,
            read_data.stats.maybe(),
            read_data.trackers.admin.maybe(),
        )
            .join()
            .filter(|(_, _, client, _, _, _)| {
                client.is_none_or(|client| client.client_type.emit_login_events())
            })
            .map(|(entity, uid, _, player, stats, admin)| {
                (
                    (*uid, PlayerInfo {
                        is_online: true,
                        is_moderator: admin.is_some(),
                        player_alias: player.alias.clone(),
                        character: stats.map(|stats| CharacterInfo {
                            name: stats.name.clone(),
                            // NOTE: hack, read docs for body::Gender for more
                            gender: stats.original_body.humanoid_gender(),
                            battle_mode: player.battle_mode,
                        }),
                        uuid: player.uuid(),
                    }),
                    (player.uuid(), entity),
                )
            })
            .unzip();
        let max_players = usize::from(read_data.settings.max_players);
        // NOTE: max_players starts as a u16, so this will not use unlimited memory even
        // if people set absurdly high values (though we should also cap the value
        // elsewhere).
        let capacity = max_players * 2;
        // List of new players to update player lists of all clients.
        //
        // Big enough that we hopefully won't have to reallocate.
        //
        // Also includes a list of logins to retry and finished_pending, since we
        // happen to update those around the same time that we update the new
        // players list.
        //
        // NOTE: stdlib mutex is more than good enough on Linux and (probably) Windows,
        // but not Mac.
        let new_players = parking_lot::Mutex::new((
            HashMap::<_, (_, _, _, _)>::with_capacity(capacity),
            Vec::with_capacity(capacity),
            Vec::with_capacity(capacity),
        ));

        // defer auth lockup
        for (entity, client) in (&read_data.entities, &mut clients).join() {
            let mut locale = None;

            let _ = super::try_recv_all(client, 0, |_, msg: ClientRegister| {
                trace!(?msg.token_or_username, "defer auth lockup");
                let pending = read_data.login_provider.verify(&msg.token_or_username);
                locale = msg.locale;
                let _ = pending_logins.insert(entity, pending);
                Ok(())
            });

            // Update locale
            if let Some(locale) = locale {
                client.locale = Some(locale);
            }
        }

        let old_player_count = player_list.len();

        // NOTE: this is just default value.
        //
        // It will be overwritten in ServerExt::update_character_data.
        let battle_mode = read_data.settings.gameplay.battle_mode.default_mode();
        let mut upgradeable_bans: EventBus<(NormalizedIpAddr, Uuid, String)> = EventBus::default();

        (
            &read_data.entities,
            &read_data.uids,
            &clients,
            !players.mask(),
            &mut pending_logins,
        )
            .join()
            // NOTE: Required because Specs has very poor work splitting for sparse joins.
            .par_bridge()
            .for_each_init(
                || (read_data.client_disconnect_events.emitter(), upgradeable_bans.emitter()),
                |(client_disconnect_emitter, upgradeable_ban_emitter), (entity, uid, client, _, pending)| {
                    prof_span!("msg::register login");
                    if let Err(e) = || -> Result<(), crate::error::Error> {
                        let extra_checks = |username: String, uuid: authc::Uuid| {
                            // We construct a few things outside the lock to reduce contention.
                            let pending_login = PendingLogin::new_success(username.clone(), uuid);
                            let player = Player::new(username, battle_mode, uuid, None);
                            let admin = editable_settings.admins.get(&uuid);
                            let player_list_update_msg = player
                                .is_valid()
                                .then_some(PlayerInfo {
                                    player_alias: player.alias.clone(),
                                    is_online: true,
                                    is_moderator: admin.is_some(),
                                    character: None, // new players will be on character select.
                                    uuid: player.uuid(),
                                })
                                .map(|player_info| {
                                    // Prepare the player list update to be sent to all clients.
                                    client.prepare(ServerGeneral::PlayerListUpdate(
                                        PlayerListUpdate::Add(*uid, player_info),
                                    ))
                                });
                            // Check if this player was already logged in before the system
                            // started.
                            let old_player = old_players_by_uuid
                            .get(&uuid)
                            .copied()
                            // We only need the old client to report an error; however, we
                            // can't assume the old player has a client (even though it would
                            // be a bit strange for them not to), so we have to remember that
                            // case.  So we grab the old client (outside the lock, to avoid
                            // contention).  We have to distinguish this from the case of a
                            // *new* player already having logged in (which we can't check
                            // until the lock is taken); in that case, we *know* the client
                            // is present, since the list is only populated by the current
                            // join (which includes the client).
                            .map(|entity| (entity, Some(clients.get(entity))));
                            // We take the lock only when necessary, and for a short duration,
                            // to avoid contention with other threads.  We need to hold the
                            // guard past the end of the login function because otherwise
                            // there's a race between when we read it and when we (potentially)
                            // write to it.
                            let guard = new_players.lock();
                            // Guard comes first in the tuple so it's dropped before the other
                            // stuff if login returns an error.
                            (
                                old_player_count + guard.0.len() >= max_players,
                                (
                                    guard,
                                    (
                                        pending_login,
                                        player,
                                        admin,
                                        player_list_update_msg,
                                        old_player,
                                    ),
                                ),
                            )
                        };

                        // Destructure new_players_guard last so it gets dropped before the other
                        // three.
                        let (
                            (pending_login, player, admin, player_list_update_msg, old_player),
                            mut new_players_guard,
                        ) = match LoginProvider::login(
                            pending,
                            client,
                            &editable_settings.admins,
                            &editable_settings.whitelist,
                            &editable_settings.banlist,
                            extra_checks,
                            |ip, uuid, username| {
                                upgradeable_ban_emitter.emit((ip, uuid, username))
                            },
                        ) {
                            None => return Ok(()),
                            Some(r) => {
                                match r {
                                    Err(e) => {
                                        new_players.lock().2.push(entity);
                                        // NOTE: Done only on error to avoid doing extra work within
                                        // the lock.
                                        trace!(?e, "pending login returned error");
                                        client_disconnect_emitter.emit(ClientDisconnectEvent(
                                            entity,
                                            common::comp::DisconnectReason::Kicked,
                                        ));
                                        client.send(Err(e))?;
                                        return Ok(());
                                    },
                                    // Swap the order of the tuple, so when it's destructured guard
                                    // is dropped first.
                                    Ok((guard, res)) => (res, guard),
                                }
                            },
                        };

                        if !client
                            .client_type
                            .is_valid_for_role(admin.map(|admin| admin.role.into()))
                        {
                            drop(new_players_guard);
                            client_disconnect_emitter.emit(ClientDisconnectEvent(
                                entity,
                                common::comp::DisconnectReason::InvalidClientType,
                            ));
                            return Ok(());
                        }

                        let (new_players_by_uuid, retries, finished_pending) =
                            &mut *new_players_guard;
                        finished_pending.push(entity);
                        // Check if the user logged in before us during this tick (this is why we
                        // need the lock held).
                        let uuid = player.uuid();
                        let old_player = old_player.map_or_else(
                            move || match new_players_by_uuid.entry(uuid) {
                                // We don't actually extract the client yet, to avoid doing extra
                                // work with the lock held.
                                hash_map::Entry::Occupied(o) => Either::Left((o.get().0, None)),
                                hash_map::Entry::Vacant(v) => Either::Right(v),
                            },
                            Either::Left,
                        );
                        let vacant_player = match old_player {
                            Either::Left((old_entity, old_client)) => {
                                if matches!(old_client, None | Some(Some(_))) {
                                    // We can't login the new client right now as the
                                    // removal of the old client and player occurs later in
                                    // the tick, so we instead setup the new login to be
                                    // processed in the next tick
                                    // Create "fake" successful pending auth and mark it to
                                    // be inserted into pending_logins at the end of this
                                    // run.
                                    retries.push((entity, pending_login));
                                    drop(new_players_guard);
                                    let old_client = old_client
                                        .flatten()
                                        .or_else(|| clients.get(old_entity))
                                        .expect(
                                            "All entries in the new player list were explicitly \
                                             joining on client",
                                        );
                                    let _ = old_client.send(ServerGeneral::Disconnect(
                                        DisconnectReason::Kicked(String::from(
                                            "You have logged in from another location.",
                                        )),
                                    ));
                                } else {
                                    drop(new_players_guard);
                                    // A player without a client is strange, so we don't really want
                                    // to retry.  Warn about this case and hope that trying to
                                    // perform the disconnect process removes the invalid player
                                    // entry.
                                    warn!(
                                        "Player without client detected for entity {:?}",
                                        old_entity
                                    );
                                }
                                // Remove old client
                                client_disconnect_emitter.emit(ClientDisconnectEvent(
                                    old_entity,
                                    common::comp::DisconnectReason::NewerLogin,
                                ));
                                return Ok(());
                            },
                            Either::Right(v) => v,
                        };

                        let Some(player_login_msg) = player_list_update_msg else {
                            drop(new_players_guard);
                            // Invalid player
                            client.send(Err(RegisterError::InvalidCharacter))?;
                            return Ok(());
                        };

                        // We know the player list didn't already contain this entity because we
                        // joined on !players, so we can assume from here that we'll definitely be
                        // adding a new player.

                        // Add to list to notify all clients of the new player
                        vacant_player.insert((
                            entity,
                            player,
                            admin,
                            client
                                .client_type
                                .emit_login_events()
                                .then_some(player_login_msg),
                        ));
                        drop(new_players_guard);
                        read_data.player_metrics.players_connected.inc();

                        // Tell the client its request was successful.
                        client.send(Ok(()))?;

                        #[cfg(feature = "plugins")]
                        let active_plugins = read_data.plugin_mgr.plugin_list();
                        #[cfg(not(feature = "plugins"))]
                        let active_plugins = Vec::default();

                        let server_descriptions = &editable_settings.server_description;
                        let description = ServerDescription {
                            motd: server_descriptions
                                .get(client.locale.as_deref())
                                .map(|d| d.motd.clone())
                                .unwrap_or_default(),
                            rules: server_descriptions
                                .get_rules(client.locale.as_deref())
                                .map(str::to_string),
                        };

                        // Send client all the tracked components currently attached to its entity
                        // as well as synced resources (currently only `TimeOfDay`)
                        debug!("Starting initial sync with client.");
                        client.send(ServerInit::GameSync {
                            // Send client their entity
                            entity_package: read_data
                                .trackers
                                .create_entity_package_with_uid(entity, *uid, None, None, None),
                            role: admin.map(|admin| admin.role.into()),
                            time_of_day: *read_data.time_of_day,
                            max_group_size: read_data.settings.max_player_group_size,
                            client_timeout: read_data.settings.client_timeout,
                            world_map: (*read_data.map).clone(),
                            recipe_book: (*read_data.recipe_book).clone(),
                            component_recipe_book: default_component_recipe_book().cloned(),
                            repair_recipe_book: default_repair_recipe_book().cloned(),
                            material_stats: (*read_data.material_stats).clone(),
                            ability_map: (*read_data.ability_map).clone(),
                            server_constants: ServerConstants {
                                day_cycle_coefficient: read_data.settings.day_cycle_coefficient(),
                            },
                            description,
                            active_plugins,
                        })?;
                        debug!("Done initial sync with client.");

                        // Send initial player list
                        client.send(ServerGeneral::PlayerListUpdate(PlayerListUpdate::Init(
                            player_list.clone(),
                        )))?;

                        Ok(())
                    }() {
                        trace!(?e, "failed to process register");
                    }
                },
            );

        let (new_players, retries, finished_pending) = new_players.into_inner();
        finished_pending.into_iter().for_each(|e| {
            // Remove all entities in finished_pending from pending_logins.
            pending_logins.remove(e);
        });

        // Insert retry attempts back into pending_logins to be processed next tick
        for (entity, pending) in retries {
            let _ = pending_logins.insert(entity, pending);
        }

        // Handle new players.
        let msgs = new_players
            .into_values()
            .filter_map(|(entity, player, admin, msg)| {
                let username = &player.alias;
                let uuid = player.uuid();
                info!(?username, "New User");
                // Add Player component to this client.
                //
                // Note that since players has been write locked for the duration of this
                // system, we know that nobody else added any players since we
                // last checked its value, and we checked that everything in
                // new_players was not already in players, so we know the insert
                // succeeds and the old entry was vacant.  Moreover, we know that all new
                // players we added have different UUIDs both from each other, and from any old
                // players, preserving the uniqueness invariant.
                players
                    .insert(entity, player)
                    .expect("The entity was joined against in the same system, so it exists");

                // Give the Admin component to the player if their name exists in
                // admin list
                if let Some(admin) = admin {
                    // We need to defer writing to the Admin storage since it's borrowed immutably
                    // by this system via TrackedStorages.
                    make_admin_emitter.emit(MakeAdminEvent {
                        entity,
                        admin: Admin(admin.role.into()),
                        uuid,
                    });
                }
                msg
            })
            .collect::<Vec<_>>();

        // Tell all clients to add the new players to the player list, in parallel.
        (players.mask(), &clients)
            .par_join()
            .for_each(|(_, client)| {
                // Send messages sequentially within each client; by the time we have enough
                // players to make parallelizing useful, we will have way more
                // players than cores.
                msgs.iter().for_each(|msg| {
                    let _ = client.send_prepared(msg);
                });
            });

        for (ip, uuid, username) in upgradeable_bans.recv_all_mut() {
            if let Err(error) = editable_settings.banlist.ban_operation(
                read_data.data_dir.as_ref(),
                chrono::Utc::now(),
                uuid,
                username,
                BanOperation::UpgradeToIpBan { ip },
                false,
            ) {
                warn!(?error, ?uuid, "Upgrading ban to IP ban failed");
            }
        }
    }
}
