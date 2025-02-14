//! Structs representing a playable Character

use crate::{comp, comp::inventory::Inventory};
use serde::{Deserialize, Serialize};

/// The limit on how many characters that a player can have
pub const MAX_CHARACTERS_PER_PLAYER: usize = 8;
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(transparent)]
pub struct CharacterId(pub i64);

pub const MAX_NAME_LENGTH: usize = 20;

/// The minimum character data we need to create a new character on the server.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    pub id: Option<CharacterId>,
    pub alias: String,
}

/// Data needed to render a single character item in the character list
/// presented during character selection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterItem {
    pub character: Character,
    pub body: comp::Body,
    pub hardcore: bool,
    pub inventory: Inventory,
    // this string changes between database representation and human readable name in server.tick
    pub location: Option<String>,
}
