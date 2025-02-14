pub mod faction;
pub mod name;
pub mod site;

use crate::data::{
    CURRENT_VERSION, Data, Nature,
    faction::Faction,
    npc::{Npc, Npcs, Profession},
    site::Site,
};
use common::{
    comp::{self, Body},
    resources::TimeOfDay,
    rtsim::{Personality, Role, WorldSettings},
    terrain::{BiomeKind, CoordinateConversions, TerrainChunkSize},
    vol::RectVolSize,
};
use rand::prelude::*;
use tracing::info;
use vek::*;
use world::{
    CONFIG, IndexRef, World,
    site::SiteKind,
    site2::{PlotKind, plot::PlotKindMeta},
};

impl Data {
    pub fn generate(settings: &WorldSettings, world: &World, index: IndexRef) -> Self {
        let mut seed = [0; 32];
        seed.iter_mut()
            .zip(&mut index.seed.to_le_bytes())
            .for_each(|(dst, src)| *dst = *src);
        let mut rng = SmallRng::from_seed(seed);

        let mut this = Self {
            version: CURRENT_VERSION,
            nature: Nature::generate(world),
            npcs: Npcs::default(),
            sites: Default::default(),
            factions: Default::default(),
            reports: Default::default(),

            tick: 0,
            time_of_day: TimeOfDay(settings.start_time),
            should_purge: false,
        };

        let initial_factions = (0..16)
            .map(|_| {
                let faction = Faction::generate(world, index, &mut rng);
                let wpos = world
                    .sim()
                    .get_size()
                    .map2(TerrainChunkSize::RECT_SIZE, |e, sz| {
                        rng.gen_range(0..(e * sz) as i32)
                    });
                (wpos, this.factions.create(faction))
            })
            .collect::<Vec<_>>();
        info!("Generated {} rtsim factions.", this.factions.len());

        // Register sites with rtsim
        for (world_site_id, _) in index.sites.iter() {
            let site = Site::generate(
                world_site_id,
                world,
                index,
                &initial_factions,
                &this.factions,
                &mut rng,
            );
            this.sites.create(site);
        }
        info!(
            "Registering {} rtsim sites from world sites.",
            this.sites.len()
        );
        // Spawn some test entities at the sites
        for (site_id, site, site2) in this.sites.iter()
        // TODO: Stupid. Only find site2 towns
        .filter_map(|(site_id, site)| Some((site_id, site, site.world_site
            .and_then(|ws| match &index.sites.get(ws).kind {
                SiteKind::Refactor(site2)
                | SiteKind::CliffTown(site2)
                | SiteKind::SavannahTown(site2)
                | SiteKind::CoastalTown(site2)
                | SiteKind::DesertCity(site2) => Some(site2),
                _ => None,
            })?)))
        {
            let Some(good_or_evil) = site
                .faction
                .and_then(|f| this.factions.get(f))
                .map(|f| f.good_or_evil)
            else {
                continue;
            };

            let rand_wpos = |rng: &mut SmallRng, matches_plot: fn(&PlotKind) -> bool| {
                let wpos2d = site2
                    .plots()
                    .filter(|plot| matches_plot(plot.kind()))
                    .choose(&mut thread_rng())
                    .map(|plot| site2.tile_center_wpos(plot.root_tile()))
                    .unwrap_or_else(|| site.wpos.map(|e| e + rng.gen_range(-10..10)));
                wpos2d
                    .map(|e| e as f32 + 0.5)
                    .with_z(world.sim().get_alt_approx(wpos2d).unwrap_or(0.0))
            };
            let random_humanoid = |rng: &mut SmallRng| {
                let species = comp::humanoid::ALL_SPECIES.choose(&mut *rng).unwrap();
                Body::Humanoid(comp::humanoid::Body::random_with(rng, species))
            };
            let matches_buildings = (|kind: &PlotKind| {
                matches!(
                    kind,
                    PlotKind::House(_)
                        | PlotKind::Workshop(_)
                        | PlotKind::AirshipDock(_)
                        | PlotKind::Tavern(_)
                        | PlotKind::Plaza
                        | PlotKind::SavannahAirshipDock(_)
                        | PlotKind::SavannahHut(_)
                        | PlotKind::SavannahWorkshop(_)
                        | PlotKind::CliffTower(_)
                        | PlotKind::DesertCityMultiPlot(_)
                        | PlotKind::DesertCityTemple(_)
                        | PlotKind::CoastalHouse(_)
                        | PlotKind::CoastalWorkshop(_)
                )
            }) as _;
            let matches_plazas = (|kind: &PlotKind| matches!(kind, PlotKind::Plaza)) as _;
            if good_or_evil {
                for _ in 0..site2.plots().len() {
                    this.npcs.create_npc(
                        Npc::new(
                            rng.gen(),
                            rand_wpos(&mut rng, matches_buildings),
                            random_humanoid(&mut rng),
                            Role::Civilised(Some(match rng.gen_range(0..20) {
                                0 => Profession::Hunter,
                                1 => Profession::Blacksmith,
                                2 => Profession::Chef,
                                3 => Profession::Alchemist,
                                5..=8 => Profession::Farmer,
                                9..=10 => Profession::Herbalist,
                                11..=16 => Profession::Guard,
                                _ => Profession::Adventurer(rng.gen_range(0..=3)),
                            })),
                        )
                        .with_faction(site.faction)
                        .with_home(site_id)
                        .with_personality(Personality::random(&mut rng)),
                    );
                }
            } else {
                for _ in 0..15 {
                    this.npcs.create_npc(
                        Npc::new(
                            rng.gen(),
                            rand_wpos(&mut rng, matches_buildings),
                            random_humanoid(&mut rng),
                            Role::Civilised(Some(Profession::Cultist)),
                        )
                        .with_personality(Personality::random_evil(&mut rng))
                        .with_faction(site.faction)
                        .with_home(site_id),
                    );
                }
            }
            // Merchants
            if good_or_evil {
                for _ in 0..(site2.plots().len() / 6) + 1 {
                    this.npcs.create_npc(
                        Npc::new(
                            rng.gen(),
                            rand_wpos(&mut rng, matches_plazas),
                            random_humanoid(&mut rng),
                            Role::Civilised(Some(Profession::Merchant)),
                        )
                        .with_home(site_id)
                        .with_personality(Personality::random_good(&mut rng)),
                    );
                }
            }

            for plot in site2
                .plots
                .values()
                .filter(|plot| matches!(plot.kind().meta(), Some(PlotKindMeta::AirshipDock { .. })))
            {
                let wpos = site2.tile_center_wpos(plot.root_tile());
                let wpos = wpos.as_().with_z(world.sim().get_surface_alt_approx(wpos))
                    + Vec3::unit_z() * 70.0;
                let vehicle_id = this.npcs.create_npc(Npc::new(
                    rng.gen(),
                    wpos,
                    Body::Ship(comp::body::ship::Body::DefaultAirship),
                    Role::Vehicle,
                ));

                let npc_id = this.npcs.create_npc(
                    Npc::new(
                        rng.gen(),
                        wpos,
                        random_humanoid(&mut rng),
                        Role::Civilised(Some(Profession::Captain)),
                    )
                    .with_home(site_id)
                    .with_personality(Personality::random_good(&mut rng)),
                );
                this.npcs
                    .mounts
                    .steer(vehicle_id, npc_id)
                    .expect("We just created these npcs");
            }
        }

        for (site_id, site) in this.sites.iter() {
            let rand_wpos = |rng: &mut SmallRng| {
                // don't spawn in buildings
                let spread_factor = rng.gen_range(-3..3) * 50;
                let spread = if spread_factor == 0 {
                    100
                } else {
                    spread_factor
                };
                let wpos2d = site.wpos.map(|e| e + spread);
                wpos2d
                    .map(|e| e as f32 + 0.5)
                    .with_z(world.sim().get_alt_approx(wpos2d).unwrap_or(0.0))
            };
            let site_kind = site.world_site.map(|ws| &index.sites.get(ws).kind);
            let Some(species) = [
                Some(comp::body::bird_large::Species::Phoenix)
                    .filter(|_| matches!(site_kind, Some(SiteKind::DwarvenMine(_)))),
                Some(comp::body::bird_large::Species::Cockatrice)
                    .filter(|_| matches!(site_kind, Some(SiteKind::Myrmidon(_)))),
                Some(comp::body::bird_large::Species::Roc)
                    .filter(|_| matches!(site_kind, Some(SiteKind::Haniwa(_)))),
                Some(comp::body::bird_large::Species::FlameWyvern)
                    .filter(|_| matches!(site_kind, Some(SiteKind::Terracotta(_)))),
                Some(comp::body::bird_large::Species::CloudWyvern)
                    .filter(|_| matches!(site_kind, Some(SiteKind::Sahagin(_)))),
                Some(comp::body::bird_large::Species::FrostWyvern)
                    .filter(|_| matches!(site_kind, Some(SiteKind::Adlet(_)))),
                Some(comp::body::bird_large::Species::SeaWyvern)
                    .filter(|_| matches!(site_kind, Some(SiteKind::ChapelSite(_)))),
                Some(comp::body::bird_large::Species::WealdWyvern)
                    .filter(|_| matches!(site_kind, Some(SiteKind::GiantTree(_)))),
            ]
            .into_iter()
            .flatten()
            .choose(&mut rng) else {
                continue;
            };

            this.npcs.create_npc(
                Npc::new(
                    rng.gen(),
                    rand_wpos(&mut rng),
                    Body::BirdLarge(comp::body::bird_large::Body::random_with(
                        &mut rng, &species,
                    )),
                    Role::Wild,
                )
                .with_home(site_id),
            );
        }

        // Spawn monsters into the world
        for _ in 0..(world.sim().map_size_lg().chunks_len() / 2usize.pow(13)).clamp(5, 1000) {
            // Try a few times to find a location that's not underwater
            if let Some((wpos, chunk)) = (0..10)
                .map(|_| world.sim().get_size().map(|sz| rng.gen_range(0..sz as i32)))
                .find_map(|pos| Some((pos, world.sim().get(pos).filter(|c| !c.is_underwater())?)))
                .map(|(pos, chunk)| {
                    let wpos2d = pos.cpos_to_wpos_center();
                    (
                        wpos2d
                            .map(|e| e as f32 + 0.5)
                            .with_z(world.sim().get_alt_approx(wpos2d).unwrap_or(0.0)),
                        chunk,
                    )
                })
            {
                let biome = chunk.get_biome();
                let Some(species) = [
                    Some(comp::body::biped_large::Species::Ogre),
                    Some(comp::body::biped_large::Species::Cyclops),
                    Some(comp::body::biped_large::Species::Wendigo)
                        .filter(|_| biome == BiomeKind::Taiga),
                    Some(comp::body::biped_large::Species::Cavetroll),
                    Some(comp::body::biped_large::Species::Mountaintroll)
                        .filter(|_| biome == BiomeKind::Mountain),
                    Some(comp::body::biped_large::Species::Swamptroll)
                        .filter(|_| biome == BiomeKind::Swamp),
                    Some(comp::body::biped_large::Species::Blueoni),
                    Some(comp::body::biped_large::Species::Redoni),
                    Some(comp::body::biped_large::Species::Tursus)
                        .filter(|_| chunk.temp < CONFIG.snow_temp),
                ]
                .into_iter()
                .flatten()
                .choose(&mut rng) else {
                    continue;
                };

                this.npcs.create_npc(Npc::new(
                    rng.gen(),
                    wpos,
                    Body::BipedLarge(comp::body::biped_large::Body::random_with(
                        &mut rng, &species,
                    )),
                    Role::Monster,
                ));
            }
        }
        // Spawn one monster Gigasfrost into the world
        // Try a few times to find a location that's not underwater
        if let Some((wpos, chunk)) = (0..100)
            .map(|_| world.sim().get_size().map(|sz| rng.gen_range(0..sz as i32)))
            .find_map(|pos| Some((pos, world.sim().get(pos).filter(|c| !c.is_underwater())?)))
            .map(|(pos, chunk)| {
                let wpos2d = pos.cpos_to_wpos_center();
                (
                    wpos2d
                        .map(|e| e as f32 + 0.5)
                        .with_z(world.sim().get_alt_approx(wpos2d).unwrap_or(0.0)),
                    chunk,
                )
            })
        {
            let species = Some(comp::body::biped_large::Species::Gigasfrost)
                .filter(|_| chunk.temp < CONFIG.snow_temp)
                .unwrap_or(comp::body::biped_large::Species::Gigasfrost);

            this.npcs.create_npc(Npc::new(
                rng.gen(),
                wpos,
                Body::BipedLarge(comp::body::biped_large::Body::random_with(
                    &mut rng, &species,
                )),
                Role::Monster,
            ));
        }

        info!("Generated {} rtsim NPCs.", this.npcs.len());

        this
    }
}
