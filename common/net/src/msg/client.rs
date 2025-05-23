use super::{PingMsg, world_msg::SiteId};
use common::{
    ViewDistances,
    character::CharacterId,
    comp::{self, AdminRole, Skill},
    event::PluginHash,
    resources::BattleMode,
    terrain::block::Block,
};
use serde::{Deserialize, Serialize};
use vek::*;

///This struct contains all messages the client might send (on different
/// streams though). It's used to verify the correctness of the state in
/// debug_assertions
#[derive(Debug, Clone)]
pub enum ClientMsg {
    ///Send on the first connection ONCE to identify client intention for
    /// server
    Type(ClientType),
    ///Send ONCE to register/auth to the server
    Register(ClientRegister),
    ///Msg that can be send ALWAYS as soon as we are registered, e.g. `Chat`
    General(ClientGeneral),
    Ping(PingMsg),
}

/*
2nd Level Enums
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientType {
    /// Regular Client like Voxygen who plays the game
    Game,
    /// A Chat-only client, which doesn't want to connect via its character
    ChatOnly,
    /// A client that is only allowed to use spectator, does not emit
    /// login/logout and player list events, and cannot use chat.
    ///
    /// Can only be used by moderators.
    SilentSpectator,
    /// A unprivileged bot, e.g. to request world information
    /// Or a privileged bot, e.g. to run admin commands used by server-cli
    Bot { privileged: bool },
}

impl ClientType {
    pub fn is_valid_for_role(&self, role: Option<AdminRole>) -> bool {
        match self {
            Self::SilentSpectator => role.is_some(),
            Self::Bot { privileged } => !privileged || role.is_some(),
            _ => true,
        }
    }

    pub fn emit_login_events(&self) -> bool { !matches!(self, Self::SilentSpectator) }

    pub fn can_spectate(&self) -> bool { matches!(self, Self::Game | Self::SilentSpectator) }

    pub fn can_enter_character(&self) -> bool { *self == Self::Game }

    pub fn can_send_message(&self) -> bool { !matches!(self, Self::SilentSpectator) }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientRegister {
    pub token_or_username: String,
    pub locale: Option<String>,
}

/// Messages sent from the client to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientGeneral {
    //Only in Character Screen
    RequestCharacterList,
    CreateCharacter {
        alias: String,
        mainhand: Option<String>,
        offhand: Option<String>,
        body: comp::Body,
        // Character will be deleted upon death if true
        hardcore: bool,
        start_site: Option<SiteId>,
    },
    DeleteCharacter(CharacterId),
    EditCharacter {
        id: CharacterId,
        alias: String,
        body: comp::Body,
    },
    Character(CharacterId, ViewDistances),
    Spectate(ViewDistances),
    //Only in game
    ControllerInputs(Box<comp::ControllerInputs>),
    ControlEvent(comp::ControlEvent),
    ControlAction(comp::ControlAction),
    SetViewDistance(ViewDistances),
    BreakBlock(Vec3<i32>),
    PlaceBlock(Vec3<i32>, Block),
    ExitInGame,
    PlayerPhysics {
        pos: comp::Pos,
        vel: comp::Vel,
        ori: comp::Ori,
        force_counter: u64,
    },
    UnlockSkill(Skill),
    RequestSiteInfo(SiteId),
    UpdateMapMarker(comp::MapMarkerChange),
    SetBattleMode(BattleMode),

    SpectatePosition(Vec3<f32>),
    //Only in Game, via terrain stream
    TerrainChunkRequest {
        key: Vec2<i32>,
    },
    LodZoneRequest {
        key: Vec2<i32>,
    },
    //Always possible
    ChatMsg(comp::Content),
    Command(String, Vec<String>),
    Terminate,
    RequestPlayerPhysics {
        server_authoritative: bool,
    },
    RequestLossyTerrainCompression {
        lossy_terrain_compression: bool,
    },
    RequestPlugins(Vec<PluginHash>),
}

impl ClientMsg {
    pub fn verify(
        &self,
        c_type: ClientType,
        registered: bool,
        presence: Option<comp::PresenceKind>,
    ) -> bool {
        match self {
            ClientMsg::Type(t) => c_type == *t,
            ClientMsg::Register(_) => !registered && presence.is_none(),
            ClientMsg::General(g) => {
                registered
                    && match g {
                        ClientGeneral::RequestCharacterList
                        | ClientGeneral::CreateCharacter { .. }
                        | ClientGeneral::EditCharacter { .. }
                        | ClientGeneral::DeleteCharacter(_) => {
                            c_type != ClientType::ChatOnly && presence.is_none()
                        },
                        ClientGeneral::Character(_, _) => {
                            c_type == ClientType::Game && presence.is_none()
                        },
                        ClientGeneral::Spectate(_) => {
                            c_type.can_spectate() && presence.is_none()
                        },
                        //Only in game
                        ClientGeneral::ControllerInputs(_)
                        | ClientGeneral::ControlEvent(_)
                        | ClientGeneral::ControlAction(_)
                        | ClientGeneral::SetViewDistance(_)
                        | ClientGeneral::BreakBlock(_)
                        | ClientGeneral::PlaceBlock(_, _)
                        | ClientGeneral::ExitInGame
                        | ClientGeneral::PlayerPhysics { .. }
                        | ClientGeneral::TerrainChunkRequest { .. }
                        | ClientGeneral::UnlockSkill(_)
                        | ClientGeneral::RequestSiteInfo(_)
                        | ClientGeneral::RequestPlayerPhysics { .. }
                        | ClientGeneral::RequestLossyTerrainCompression { .. }
                        | ClientGeneral::UpdateMapMarker(_)
                        | ClientGeneral::SetBattleMode(_) => {
                            c_type == ClientType::Game && presence.is_some()
                        },
                        ClientGeneral::SpectatePosition(_) => {
                            c_type.can_spectate() && presence.is_some()
                        },
                        ClientGeneral::ChatMsg(_) => {
                            c_type.can_send_message()
                        },
                        //Always possible
                        ClientGeneral::Command(_, _)
                        | ClientGeneral::Terminate
                        // LodZoneRequest is required by the char select screen
                        | ClientGeneral::LodZoneRequest { .. } => true,
                        | ClientGeneral::RequestPlugins(_) => true,
                    }
            },
            ClientMsg::Ping(_) => true,
        }
    }
}

/*
end of 2nd level Enums
*/

impl From<ClientType> for ClientMsg {
    fn from(other: ClientType) -> ClientMsg { ClientMsg::Type(other) }
}

impl From<ClientRegister> for ClientMsg {
    fn from(other: ClientRegister) -> ClientMsg { ClientMsg::Register(other) }
}

impl From<ClientGeneral> for ClientMsg {
    fn from(other: ClientGeneral) -> ClientMsg { ClientMsg::General(other) }
}

impl From<PingMsg> for ClientMsg {
    fn from(other: PingMsg) -> ClientMsg { ClientMsg::Ping(other) }
}
