use libtw2_gamenet::msg::Game;
use libtw2_gamenet::msg::SystemOrGame;
use libtw2_gamenet::msg::game::ClSay;
use libtw2_gamenet::msg;
use libtw2_packer::with_packer;
use libtw2_packer::Unpacker;
use warn::Ignore;

use super::Proxy as ProxyTrait;
use super::Send;

pub const SERVER_VERSION: i32 = 18030;

pub struct Proxy {
    client_version: i32,
}

impl Proxy {
    pub fn new(client_version: i32) -> Option<Proxy> {
        if client_version >= SERVER_VERSION {
            return None;
        }
        Some(Proxy {
            client_version,
        })
    }
}

impl ProxyTrait for Proxy {
    fn translate_client_packet(&mut self, packet: &[u8], unreliable: bool, _to_client: &mut dyn Send, to_server: &mut dyn Send) -> bool {
        match msg::decode(&mut Ignore, &mut Unpacker::new(packet)) {
            Ok(SystemOrGame::Game(Game::ClSay(ClSay { team, message: _ }))) if self.client_version >= 0 => {
                with_packer(to_server.send(unreliable), |p| Game::from(ClSay { team, message: "foobar2".as_bytes() }).encode(p).unwrap());
                false
            }
            _ => true,
        }
    }
}
