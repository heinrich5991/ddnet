use arrayvec::ArrayString;
use libtw2_gamenet::enums::FLAG_TAKEN;
use libtw2_gamenet::msg::Game;
use libtw2_gamenet::msg::MessageId;
use libtw2_gamenet::msg::SystemOrGame;
use libtw2_gamenet::msg::game;
use libtw2_gamenet::snap_obj;
use libtw2_packer::IntUnpacker;
use libtw2_packer::Unpacker;
use libtw2_packer::with_packer;
use libtw2_snapshot::Snap;
use libtw2_snapshot::snap;
use libtw2_snapshot::format::Item as SnapItem;
use libtw2_snapshot::format::TypeId;
use std::fmt::Write as _;
use std::mem;
use std::str;
use warn::Panic;

use super::Proxy as ProxyTrait;
use super::Send;

struct Invalid;
struct UnmappedClientId;

enum Error {
    Invalid,
    UnmappedClientId,
}

impl From<libtw2_gamenet::Error> for Invalid {
    fn from(_: libtw2_gamenet::Error) -> Invalid {
        Invalid
    }
}

impl From<libtw2_gamenet::Error> for Error {
    fn from(_: libtw2_gamenet::Error) -> Error {
        Error::Invalid
    }
}

impl From<UnmappedClientId> for Error {
    fn from(UnmappedClientId: UnmappedClientId) -> Error {
        Error::UnmappedClientId
    }
}

pub struct Proxy {
    client_max_clients: i32,
    client_to_server_id: Vec<Option<i32>>,
    server_to_client_id: Vec<Option<i32>>,
    snap_buf: Snap,
    snap_builder: snap::Builder,
    snap_write_buf: Vec<i32>,
}

impl Proxy {
    pub fn new() -> Proxy {
        Proxy {
            client_max_clients: 64,
            client_to_server_id: (0..64).rev().map(Some).collect(),
            server_to_client_id: (0..64).rev().map(Some).collect(),
            snap_buf: Snap::empty(),
            snap_builder: snap::Builder::new(),
            snap_write_buf: Vec::new(),
        }
    }
    fn client_to_server_id(&self, client_id: i32) -> Result<i32, Invalid> {
        let client_id: usize = client_id.try_into().map_err(|_| Invalid)?;
        self.client_to_server_id.get(client_id).ok_or(Invalid)?.ok_or(Invalid)
    }
    fn server_to_client_id(&self, client_id: i32) -> Result<i32, UnmappedClientId> {
        let client_id: usize = client_id.try_into().expect("invalid negative client ID");
        self.server_to_client_id.get(client_id).ok_or(UnmappedClientId)?.ok_or(UnmappedClientId)
    }
    fn translate_client_to_server_id(&self, client_id: &mut i32) -> Result<(), Invalid> {
        *client_id = self.client_to_server_id(*client_id)?;
        Ok(())
    }
    fn translate_server_to_client_id(&self, client_id: &mut i32) -> Result<(), UnmappedClientId> {
        *client_id = self.server_to_client_id(*client_id)?;
        Ok(())
    }
    fn translate_client_packet_impl(&mut self, packet: &[u8], unreliable: bool, to_server: &mut dyn Send) -> Result<bool, Invalid> {
        let mut str_buf: ArrayString<[u8; 32]>;
        let unpacker = &mut Unpacker::new(packet);
        let translated: Game = match SystemOrGame::decode_id(&mut Panic, unpacker)? {
            SystemOrGame::Game(MessageId::Ordinal(game::CL_SET_SPECTATOR_MODE)) => {
                let mut msg = game::ClSetSpectatorMode::decode(&mut Panic, unpacker)?;
                if msg.spectator_id < 0 {
                    return Ok(true);
                }
                self.translate_client_to_server_id(&mut msg.spectator_id)?;
                msg.into()
            }
            SystemOrGame::Game(MessageId::Ordinal(game::CL_CALL_VOTE)) => {
                let mut msg = game::ClCallVote::decode(&mut Panic, unpacker)?;
                if msg.type_ != b"kick" && msg.type_ != b"spectate" {
                    return Ok(true);
                }
                let client_id: i32 = str::from_utf8(msg.value).map_err(|_| Invalid)?.parse().map_err(|_| Invalid)?;
                let client_id = self.client_to_server_id(client_id)?;
                str_buf = ArrayString::new();
                write!(str_buf, "{}", client_id).unwrap();
                msg.value = str_buf.as_bytes();
                msg.into()
            }
            _ => return Ok(true),
        };
        with_packer(to_server.send(unreliable), |p| translated.encode(p)).unwrap();
        Ok(false)
    }
    fn translate_server_packet_impl(&mut self, packet: &[u8], unreliable: bool, to_client: &mut dyn Send) -> Result<bool, Error> {
        let unpacker = &mut Unpacker::new(packet);
        let translated: Game = match SystemOrGame::decode_id(&mut Panic, unpacker)? {
            SystemOrGame::Game(MessageId::Ordinal(game::SV_CHAT)) => {
                let mut msg = game::SvChat::decode(&mut Panic, unpacker)?;
                if msg.client_id < 0 {
                    return Ok(true);
                }
                // TODO: don't drop messages
                self.translate_server_to_client_id(&mut msg.client_id)?;
                msg.into()
            }
            SystemOrGame::Game(MessageId::Ordinal(game::SV_KILL_MSG)) => {
                let mut msg = game::SvKillMsg::decode(&mut Panic, unpacker)?;
                self.translate_server_to_client_id(&mut msg.killer)?;
                self.translate_server_to_client_id(&mut msg.victim)?;
                msg.into()
            }
            SystemOrGame::Game(MessageId::Ordinal(game::SV_EMOTICON)) => {
                let mut msg = game::SvEmoticon::decode(&mut Panic, unpacker)?;
                self.translate_server_to_client_id(&mut msg.client_id)?;
                msg.into()
            }
            SystemOrGame::Game(MessageId::Ordinal(game::SV_VOTE_STATUS)) => {
                let mut msg = game::SvVoteStatus::decode(&mut Panic, unpacker)?;
                if msg.total > self.client_max_clients {
                    // TODO: better approximations?
                    msg.yes = ((msg.yes * self.client_max_clients) as f32 / msg.total as f32).round() as i32;
                    msg.no = ((msg.no * self.client_max_clients) as f32 / msg.total as f32).round() as i32;
                    msg.total = self.client_max_clients;
                    msg.pass = msg.total - msg.yes - msg.no;
                }
                msg.into()
            }
            SystemOrGame::Game(MessageId::Uuid(game::SV_KILL_MSG_TEAM)) => {
                let mut msg = game::SvKillMsgTeam::decode(&mut Panic, unpacker)?;
                self.translate_server_to_client_id(&mut msg.first)?;
                msg.into()
            }
            _ => return Ok(true),
        };
        with_packer(to_client.send(unreliable), |p| translated.encode(p)).unwrap();
        Ok(false)
    }
    fn translate_server_snap_item(&self, builder: &mut snap::Builder, type_id: TypeId, id: u16, data: &[i32]) -> Result<bool, Error> {
        let mut unpacker = IntUnpacker::new(data);
        match type_id {
            TypeId::Ordinal(snap_obj::GAME_INFO) => {
                let mut obj = snap_obj::GameInfo::decode(&mut Panic, &mut unpacker)?;
                obj.time_limit = 837;
                obj.score_limit = 1337;
                builder.add_item(type_id, id, obj.encode()).expect("TODO");
                return Ok(false);
            }
            TypeId::Ordinal(snap_obj::GAME_DATA) => {
                let mut obj = snap_obj::GameData::decode(&mut Panic, &mut unpacker)?;
                if obj.flag_carrier_red >= 0 {
                    obj.flag_carrier_red = self.server_to_client_id(obj.flag_carrier_red).unwrap_or(FLAG_TAKEN);
                }
                if obj.flag_carrier_blue >= 0 {
                    obj.flag_carrier_blue = self.server_to_client_id(obj.flag_carrier_blue).unwrap_or(FLAG_TAKEN);
                }
                builder.add_item(type_id, id, obj.encode()).expect("TODO");
                return Ok(false);
            }
            TypeId::Ordinal(snap_obj::CHARACTER_CORE) => {
                unreachable!();
            }
            TypeId::Ordinal(snap_obj::CHARACTER) => {
                let id: u16 = self.server_to_client_id(id.into())?.try_into().unwrap();
                let mut obj = snap_obj::Character::decode(&mut Panic, &mut unpacker)?;
                if obj.character_core.hooked_player >= 0 {
                    obj.character_core.hooked_player = self.server_to_client_id(obj.character_core.hooked_player).unwrap_or(-1);
                }
                builder.add_item(type_id, id, obj.encode()).expect("TODO");
                return Ok(false);
            }
            TypeId::Ordinal(snap_obj::PLAYER_INFO) => {
                let id: u16 = self.server_to_client_id(id.into())?.try_into().unwrap();
                let mut obj = snap_obj::PlayerInfo::decode(&mut Panic, &mut unpacker)?;
                self.translate_server_to_client_id(&mut obj.client_id)?;
                builder.add_item(type_id, id, obj.encode()).expect("TODO");
                return Ok(false);
            }
            TypeId::Ordinal(snap_obj::CLIENT_INFO) => {
                let id: u16 = self.server_to_client_id(id.into())?.try_into().unwrap();
                builder.add_item(type_id, id, data).expect("TODO");
                return Ok(false);
            }
            TypeId::Ordinal(snap_obj::SPECTATOR_INFO) => {
                let mut obj = snap_obj::SpectatorInfo::decode(&mut Panic, &mut unpacker)?;
                if obj.spectator_id >= 0 {
                    obj.spectator_id = self.server_to_client_id(obj.spectator_id).unwrap_or(-1);
                }
                builder.add_item(type_id, id, obj.encode()).expect("TODO");
                return Ok(false);
            }
            TypeId::Uuid(snap_obj::DDNET_PLAYER) => {
                let id: u16 = self.server_to_client_id(id.into())?.try_into().unwrap();
                builder.add_item(type_id, id, data).expect("TODO");
                return Ok(false);
            }
            TypeId::Uuid(snap_obj::DDNET_LASER) => {
                let mut obj = snap_obj::DdnetLaser::decode(&mut Panic, &mut unpacker)?;
                if obj.owner >= 0 {
                    obj.owner = self.server_to_client_id(obj.owner).unwrap_or(-1);
                }
                builder.add_item(type_id, id, data).expect("TODO");
                return Ok(false);
            }
            TypeId::Uuid(snap_obj::DDNET_PROJECTILE) => {
                let mut obj = snap_obj::DdnetProjectile::decode(&mut Panic, &mut unpacker)?;
                if obj.owner >= 0 {
                    obj.owner = self.server_to_client_id(obj.owner).unwrap_or(-1);
                }
                builder.add_item(type_id, id, data).expect("TODO");
                return Ok(false);
            }
            TypeId::Ordinal(snap_obj::DEATH) => {
                let mut obj = snap_obj::Death::decode(&mut Panic, &mut unpacker)?;
                self.translate_server_to_client_id(&mut obj.client_id)?;
                builder.add_item(type_id, id, data).expect("TODO");
                return Ok(false);
            }
            TypeId::Uuid(snap_obj::SPEC_CHAR) => {
                let id: u16 = self.server_to_client_id(id.into())?.try_into().unwrap();
                builder.add_item(type_id, id, data).expect("TODO");
                return Ok(false);
            }
            _ => {}
        }
        Ok(true)
    }
}

impl ProxyTrait for Proxy {
    fn translate_client_packet(&mut self, packet: &[u8], unreliable: bool, _to_client: &mut dyn Send, to_server: &mut dyn Send) -> bool {
        match self.translate_client_packet_impl(packet, unreliable, to_server) {
            Ok(result) => result,
            Err(Invalid) => {
                todo!("dropping invalid message from client: {:?}", packet);
                // false
            }
        }
    }
    fn translate_server_packet(&mut self, packet: &[u8], unreliable: bool, to_client: &mut dyn Send, _to_server: &mut dyn Send) -> bool {
        match self.translate_server_packet_impl(packet, unreliable, to_client) {
            Ok(result) => result,
            Err(Error::Invalid) => panic!("invalid message from server: {:?}", packet),
            Err(Error::UnmappedClientId) => false,
        }
    }
    fn translate_server_snap(&mut self, buffer: &mut [i32], snap_size: usize) -> usize {
        self.snap_buf.read_from_ints(&mut Panic, &buffer[..snap_size]).unwrap();
        let mut builder = mem::take(&mut self.snap_builder);
        for SnapItem { type_id, id, data } in self.snap_buf.items() {
            match self.translate_server_snap_item(&mut builder, type_id, id, data) {
                Ok(true) => builder.add_item(type_id, id, data).expect("TODO"),
                Ok(false) => {}
                Err(Error::Invalid) => panic!("invalid snapshot item: {} {} {:?}", type_id, id, data),
                Err(Error::UnmappedClientId) => {}
            }
        }
        let snap = builder.finish();
        let len = snap.write_to_ints(&mut self.snap_write_buf, buffer).expect("TODO").len();
        self.snap_builder = snap.recycle();
        len
    }
}
