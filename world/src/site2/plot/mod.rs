mod adlet;
mod airship_dock;
mod bridge;
mod camp;
mod castle;
mod citadel;
mod cliff_tower;
mod coastal_house;
mod coastal_workshop;
mod cultist;
mod desert_city_arena;
mod desert_city_multiplot;
mod desert_city_temple;
pub mod dungeon;
mod dwarven_mine;
mod giant_tree;
mod glider_finish;
mod glider_platform;
mod glider_ring;
mod gnarling;
mod haniwa;
mod house;
mod jungle_ruin;
mod myrmidon_arena;
mod myrmidon_house;
mod pirate_hideout;
mod rock_circle;
mod sahagin;
mod savannah_hut;
mod savannah_pit;
mod savannah_workshop;
mod sea_chapel;
pub mod tavern;
mod terracotta_house;
mod terracotta_palace;
mod terracotta_yard;
mod troll_cave;
mod vampire_castle;
mod workshop;

pub use self::{
    adlet::AdletStronghold, airship_dock::AirshipDock, bridge::Bridge, camp::Camp, castle::Castle,
    citadel::Citadel, cliff_tower::CliffTower, coastal_house::CoastalHouse,
    coastal_workshop::CoastalWorkshop, cultist::Cultist, desert_city_arena::DesertCityArena,
    desert_city_multiplot::DesertCityMultiPlot, desert_city_temple::DesertCityTemple,
    dungeon::Dungeon, dwarven_mine::DwarvenMine, giant_tree::GiantTree,
    glider_finish::GliderFinish, glider_platform::GliderPlatform, glider_ring::GliderRing,
    gnarling::GnarlingFortification, haniwa::Haniwa, house::House, jungle_ruin::JungleRuin,
    myrmidon_arena::MyrmidonArena, myrmidon_house::MyrmidonHouse, pirate_hideout::PirateHideout,
    rock_circle::RockCircle, sahagin::Sahagin, savannah_hut::SavannahHut,
    savannah_pit::SavannahPit, savannah_workshop::SavannahWorkshop, sea_chapel::SeaChapel,
    tavern::Tavern, terracotta_house::TerracottaHouse, terracotta_palace::TerracottaPalace,
    terracotta_yard::TerracottaYard, troll_cave::TrollCave, vampire_castle::VampireCastle,
    workshop::Workshop,
};

use super::*;
use crate::util::DHashSet;
use common::path::Path;
use vek::*;

pub struct Plot {
    pub(crate) kind: PlotKind,
    pub(crate) root_tile: Vec2<i32>,
    pub(crate) tiles: DHashSet<Vec2<i32>>,
    pub(crate) seed: u32,
}

impl Plot {
    pub fn find_bounds(&self) -> Aabr<i32> {
        self.tiles
            .iter()
            .fold(Aabr::new_empty(self.root_tile), |b, t| {
                b.expanded_to_contain_point(*t)
            })
    }

    pub fn z_range(&self) -> Option<Range<i32>> {
        match &self.kind {
            PlotKind::House(house) => Some(house.z_range()),
            _ => None,
        }
    }

    pub fn kind(&self) -> &PlotKind { &self.kind }

    pub fn root_tile(&self) -> Vec2<i32> { self.root_tile }

    pub fn tiles(&self) -> impl ExactSizeIterator<Item = Vec2<i32>> + '_ {
        self.tiles.iter().copied()
    }
}

pub enum PlotKind {
    House(House),
    AirshipDock(AirshipDock),
    GliderRing(GliderRing),
    GliderPlatform(GliderPlatform),
    GliderFinish(GliderFinish),
    Tavern(Tavern),
    CoastalHouse(CoastalHouse),
    CoastalWorkshop(CoastalWorkshop),
    Workshop(Workshop),
    DesertCityMultiPlot(DesertCityMultiPlot),
    DesertCityTemple(DesertCityTemple),
    DesertCityArena(DesertCityArena),
    SeaChapel(SeaChapel),
    JungleRuin(JungleRuin),
    Plaza,
    Castle(Castle),
    Cultist(Cultist),
    Road(Path<Vec2<i32>>),
    Dungeon(Dungeon),
    Gnarling(GnarlingFortification),
    Adlet(AdletStronghold),
    Haniwa(Haniwa),
    GiantTree(GiantTree),
    CliffTower(CliffTower),
    Sahagin(Sahagin),
    Citadel(Citadel),
    SavannahPit(SavannahPit),
    SavannahHut(SavannahHut),
    SavannahWorkshop(SavannahWorkshop),
    Bridge(Bridge),
    PirateHideout(PirateHideout),
    RockCircle(RockCircle),
    TrollCave(TrollCave),
    Camp(Camp),
    DwarvenMine(DwarvenMine),
    TerracottaPalace(TerracottaPalace),
    TerracottaHouse(TerracottaHouse),
    TerracottaYard(TerracottaYard),
    VampireCastle(VampireCastle),
    MyrmidonArena(MyrmidonArena),
    MyrmidonHouse(MyrmidonHouse),
}
