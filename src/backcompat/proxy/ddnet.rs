use libtw2_gamenet::msg::System;
use libtw2_gamenet::msg::Game;
use libtw2_gamenet::msg::SystemOrGame;
use libtw2_gamenet::msg::system::ClientVersion;
use libtw2_gamenet::msg::system;
use libtw2_gamenet::msg::game;
use libtw2_gamenet::msg;
use libtw2_packer::with_packer;
use libtw2_packer::Unpacker;
use warn::Ignore;

use super::Proxy as ProxyTrait;
use super::Send;

pub const SERVER_VERSION: i32 = 18030;

pub struct Proxy {
    client_version: Option<i32>,
}

impl Proxy {
    pub fn new() -> Proxy {
        Proxy {
            client_version: None,
        }
    }
}

impl ProxyTrait for Proxy {
    fn translate_client_packet(&mut self, packet: &[u8], unreliable: bool, _to_client: &mut dyn Send, to_server: &mut dyn Send) -> bool {
        /*
        let unpacker = &mut Unpacker::new(packet);
        let msg_id = match SystemOrGame::decode_id(&mut Unpacker::new(packet)) {
            Ok(id) => id,
            Err(_) => return false,
        };
        if self.client_version.is_none() {
            if matches!(,
                SystemOrGame::System(MessageId::Uuid(system::CLIENT_VERSION)) |
                SystemOrGame::Game(MessageId::Ordinal(game::CL_IS_DDNET_LEGACY))
            ) {
                match msg::decode(&mut Ignore, &mut Unpacker::new(packet)) {
                    Ok(SystemOrGame::Game(Game::ClientVersion(ClientVersion {}))) => {
                        false
                    }
                    _ => true,
                }
            }
        }

        match msg::decode(&mut Ignore, &mut Unpacker::new(packet)) {
            Ok(SystemOrGame::Game(Game::ClientVersion(ClientVersion { team, message: _ }))) => {
                false
            }
            _ => true,
        }
        */
        true
    }
}
