mod ui;

use crate::{
    Direction, GlobalState, PlayState, PlayStateResult, hud,
    menu::{
        main::{get_client_msg_error, rand_bg_image_spec},
        server_info::ServerInfoState,
    },
    render::{Drawer, GlobalsBindGroup},
    scene::simple::{self as scene, Scene},
    session::SessionState,
    settings::Settings,
    window::Event as WinEvent,
};
use client::{self, Client};
use common::{comp, event::UpdateCharacterMetadata, resources::DeltaTime};
use common_base::span;
#[cfg(feature = "plugins")]
use common_state::plugin::PluginMgr;
use specs::WorldExt;
use std::{cell::RefCell, rc::Rc};
use tracing::error;
use ui::CharSelectionUi;

pub struct CharSelectionState {
    char_selection_ui: CharSelectionUi,
    client: Rc<RefCell<Client>>,
    persisted_state: Rc<RefCell<hud::PersistedHudState>>,
    scene: Scene,
}

impl CharSelectionState {
    /// Create a new `CharSelectionState`.
    pub fn new(
        global_state: &mut GlobalState,
        client: Rc<RefCell<Client>>,
        persisted_state: Rc<RefCell<hud::PersistedHudState>>,
    ) -> Self {
        let sprite_render_context = (global_state.lazy_init)(global_state.window.renderer_mut());
        let scene = Scene::new(
            global_state.window.renderer_mut(),
            &mut client.borrow_mut(),
            &global_state.settings,
            sprite_render_context,
        );
        let char_selection_ui = CharSelectionUi::new(global_state, &client.borrow());

        Self {
            char_selection_ui,
            client,
            persisted_state,
            scene,
        }
    }

    fn get_humanoid_body_inventory<'a>(
        char_selection_ui: &'a CharSelectionUi,
        client: &'a Client,
    ) -> (
        Option<comp::humanoid::Body>,
        Option<&'a comp::inventory::Inventory>,
    ) {
        char_selection_ui
            .display_body_inventory(&client.character_list().characters)
            .map(|(body, inventory)| {
                (
                    match body {
                        comp::Body::Humanoid(body) => Some(body),
                        _ => None,
                    },
                    Some(inventory),
                )
            })
            .unwrap_or_default()
    }

    pub fn client(&self) -> &RefCell<Client> { &self.client }
}

impl PlayState for CharSelectionState {
    fn enter(&mut self, global_state: &mut GlobalState, _: Direction) {
        // Load the player's character list
        if !self.client.borrow().are_plugins_missing() {
            self.client.borrow_mut().load_character_list();
        }

        // Updated localization in case the selected language was changed
        self.char_selection_ui.update_language(global_state.i18n);
        // Set scale mode in case it was change
        self.char_selection_ui
            .set_scale_mode(global_state.settings.interface.ui_scale);

        // Clear shadow textures since we don't render to them here
        global_state.clear_shadows_next_frame = true;

        #[cfg(feature = "discord")]
        global_state.discord.enter_character_selection();
    }

    fn tick(&mut self, global_state: &mut GlobalState, events: Vec<WinEvent>) -> PlayStateResult {
        span!(_guard, "tick", "<CharSelectionState as PlayState>::tick");
        let client_registered = {
            let client = self.client.borrow();
            client.registered()
        };
        if client_registered {
            // Handle window events
            for event in events {
                if self.char_selection_ui.handle_event(event.clone()) {
                    continue;
                }
                match event {
                    WinEvent::Close => {
                        return PlayStateResult::Shutdown;
                    },
                    // Pass all other events to the scene
                    event => {
                        self.scene.handle_input_event(event);
                    }, // TODO: Do something if the event wasn't handled?
                }
            }

            // Maintain the UI.
            let events = self
                .char_selection_ui
                .maintain(global_state, &self.client.borrow());

            for event in events {
                match event {
                    ui::Event::Logout => {
                        return PlayStateResult::Pop;
                    },
                    ui::Event::AddCharacter {
                        alias,
                        mainhand,
                        offhand,
                        body,
                        hardcore,
                        start_site,
                    } => {
                        self.client
                            .borrow_mut()
                            .create_character(alias, mainhand, offhand, body, hardcore, start_site);
                    },
                    ui::Event::EditCharacter {
                        alias,
                        character_id,
                        body,
                    } => {
                        self.client
                            .borrow_mut()
                            .edit_character(alias, character_id, body);
                    },
                    ui::Event::DeleteCharacter(character_id) => {
                        self.client.borrow_mut().delete_character(character_id);
                    },
                    ui::Event::Play(character_id) => {
                        let mut c = self.client.borrow_mut();
                        let graphics = &global_state.settings.graphics;
                        c.request_character(character_id, common::ViewDistances {
                            terrain: graphics.terrain_view_distance,
                            entity: graphics.entity_view_distance,
                        });
                    },
                    ui::Event::Spectate => {
                        {
                            let mut c = self.client.borrow_mut();
                            c.request_spectate(global_state.settings.graphics.view_distances());
                        }
                        return PlayStateResult::Switch(Box::new(SessionState::new(
                            global_state,
                            UpdateCharacterMetadata::default(),
                            Rc::clone(&self.client),
                            Rc::clone(&self.persisted_state),
                        )));
                    },
                    ui::Event::ShowRules => {
                        let client = self.client.borrow();

                        let server_info = client.server_info().clone();
                        let server_description = client.server_description().clone();

                        drop(client);

                        let char_select = CharSelectionState::new(
                            global_state,
                            Rc::clone(&self.client),
                            Rc::clone(&self.persisted_state),
                        );

                        let new_state = ServerInfoState::try_from_server_info(
                            global_state,
                            rand_bg_image_spec(),
                            char_select,
                            server_info,
                            server_description,
                            true,
                        )
                        .map(|s| Box::new(s) as _)
                        .unwrap_or_else(|s| Box::new(s) as _);

                        return PlayStateResult::Switch(new_state);
                    },
                    ui::Event::ClearCharacterListError => {
                        self.char_selection_ui.error = None;
                    },
                    ui::Event::SelectCharacter(selected) => {
                        let client = self.client.borrow();
                        let server_name = &client.server_info().name;
                        // Select newly created character
                        global_state
                            .profile
                            .set_selected_character(server_name, selected);
                        global_state
                            .profile
                            .save_to_file_warn(&global_state.config_dir);
                    },
                }
            }

            // Maintain the scene.
            {
                let client = self.client.borrow();
                let (humanoid_body, loadout) =
                    Self::get_humanoid_body_inventory(&self.char_selection_ui, &client);

                // Maintain the scene.
                let scene_data = scene::SceneData {
                    time: client.state().get_time(),
                    delta_time: client.state().ecs().read_resource::<DeltaTime>().0,
                    tick: client.get_tick(),
                    slow_job_pool: &client.state().slow_job_pool(),
                    body: humanoid_body,
                    gamma: global_state.settings.graphics.gamma,
                    exposure: global_state.settings.graphics.exposure,
                    ambiance: global_state.settings.graphics.ambiance,
                    mouse_smoothing: global_state.settings.gameplay.smooth_pan_enable,
                    figure_lod_render_distance: global_state
                        .settings
                        .graphics
                        .figure_lod_render_distance
                        as f32,
                };

                self.scene.maintain(
                    global_state.window.renderer_mut(),
                    scene_data,
                    loadout,
                    &client,
                );
            }

            // Tick the client (currently only to keep the connection alive).
            let localized_strings = &global_state.i18n.read();

            let res = self
                .client
                .borrow_mut()
                .tick(comp::ControllerInputs::default(), global_state.clock.dt());
            match res {
                Ok(events) => {
                    let mut join_metadata = None;
                    for event in events {
                        match event {
                            client::Event::SetViewDistance(_vd) => {},
                            client::Event::Disconnect => {
                                global_state.info_message = Some(
                                    localized_strings
                                        .get_msg("main-login-server_shut_down")
                                        .into_owned(),
                                );
                                return PlayStateResult::Pop;
                            },
                            client::Event::Chat(m) => self
                                .persisted_state
                                .borrow_mut()
                                .message_backlog
                                .new_message(&self.client.borrow(), &global_state.profile, m),
                            client::Event::MapMarker(marker_event) => self
                                .persisted_state
                                .borrow_mut()
                                .location_markers
                                .update(marker_event),
                            client::Event::CharacterCreated(character_id) => {
                                self.char_selection_ui.select_character(character_id);
                            },
                            client::Event::CharacterError(error) => {
                                self.char_selection_ui.display_error(error);
                            },
                            client::Event::CharacterJoined(metadata) => {
                                join_metadata = Some(metadata);
                            },
                            #[cfg_attr(not(feature = "plugins"), expect(unused_variables))]
                            client::Event::PluginDataReceived(data) => {
                                #[cfg(feature = "plugins")]
                                {
                                    tracing::info!("plugin data {}", data.len());
                                    let mut client = self.client.borrow_mut();
                                    let hash = client
                                        .state()
                                        .ecs()
                                        .write_resource::<PluginMgr>()
                                        .cache_server_plugin(&global_state.config_dir, data);
                                    match hash {
                                        Ok(hash) => {
                                            if client.plugin_received(hash) == 0 {
                                                // now load characters (plugins might contain items)
                                                client.load_character_list();
                                            }
                                        },
                                        Err(e) => tracing::error!(?e, "cache_server_plugin"),
                                    }
                                }
                            },
                            // TODO: See if we should handle StartSpectate here instead.
                            _ => {},
                        }
                    }

                    if let Some(metadata) = join_metadata {
                        return PlayStateResult::Switch(Box::new(SessionState::new(
                            global_state,
                            metadata,
                            Rc::clone(&self.client),
                            Rc::clone(&self.persisted_state),
                        )));
                    }
                },
                Err(err) => {
                    error!(?err, "[char_selection] Failed to tick the client");
                    global_state.info_message =
                        Some(get_client_msg_error(err, None, &global_state.i18n.read()));
                    return PlayStateResult::Pop;
                },
            }

            // TODO: make sure rendering is not relying on cleaned up stuff
            self.client.borrow_mut().cleanup();

            PlayStateResult::Continue
        } else {
            error!("Client not in pending, or registered state. Popping char selection play state");
            // TODO set global_state.info_message
            PlayStateResult::Pop
        }
    }

    fn name(&self) -> &'static str { "Character Selection" }

    fn capped_fps(&self) -> bool { true }

    fn globals_bind_group(&self) -> &GlobalsBindGroup { self.scene.global_bind_group() }

    fn render(&self, drawer: &mut Drawer<'_>, _: &Settings) {
        let client = self.client.borrow();
        let (humanoid_body, loadout) =
            Self::get_humanoid_body_inventory(&self.char_selection_ui, &client);

        if let Some(mut first_pass) = drawer.first_pass() {
            self.scene
                .render(&mut first_pass, client.get_tick(), humanoid_body, loadout);
        }

        if let Some(mut volumetric_pass) = drawer.volumetric_pass() {
            // Clouds
            volumetric_pass.draw_clouds();
        }
        // Bloom (does nothing if bloom is disabled)
        drawer.run_bloom_passes();
        // PostProcess and UI
        let mut third_pass = drawer.third_pass();
        third_pass.draw_postprocess();
        // Draw the UI to the screen.
        if let Some(mut ui_drawer) = third_pass.draw_ui() {
            self.char_selection_ui.render(&mut ui_drawer);
        };
    }

    fn egui_enabled(&self) -> bool { false }
}
