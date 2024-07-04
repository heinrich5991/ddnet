use arrayvec::ArrayVec;

mod ddnet;

pub trait Send {
    fn send(&mut self, unreliable: bool) -> &mut ArrayVec<[u8; 2048]>;
}

pub trait Proxy {
    fn translate_client_packet(&mut self, packet: &[u8], unreliable: bool, to_client: &mut dyn Send, to_server: &mut dyn Send) -> bool {
        let _ = (packet, unreliable, to_client, to_server);
        true
    }
    fn translate_server_packet(&mut self, packet: &[u8], unreliable: bool, to_client: &mut dyn Send, to_server: &mut dyn Send) -> bool {
        let _ = (packet, unreliable, to_client, to_server);
        true
    }
    fn translate_server_snap(&mut self, buffer: &mut [i32], snap_size: usize) -> usize {
        assert!(snap_size <= buffer.len());
        snap_size
    }
}

pub fn create_ddnet(client_version: i32) -> Option<Box<dyn Proxy>> {
    self::ddnet::Proxy::new(client_version).map(|p| Box::new(p) as Box<_>)
}
