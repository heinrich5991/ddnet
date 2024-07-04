#![allow(nonstandard_style)]

use arrayvec::ArrayVec;
use cxx::UniquePtr;
use ddnet_backcompat_proxy::Proxy;
use std::collections::VecDeque;

use self::ffi::SendFunction;
use self::ffi::NetChunk;

pub const NETSENDFLAG_VITAL: i32 = 1; // Move to engine.

#[cxx::bridge]
mod ffi {
    // TODO: move to engine-shared?
    #[cxx_name = "CNetChunk2"]
    struct NetChunk<'a> {
        #[cxx_name = "m_ClientId"]
        client_id: i32,
        #[cxx_name = "m_Flags"]
        flags: i32,
        #[cxx_name = "m_pData"]
        data: &'a [u8],
    }

    unsafe extern "C++" {
        include!("backcompat/hacks/send.h");

        #[cxx_name = "CSendFunction"]
        type SendFunction;
        #[cxx_name = "Send"]
        fn send(self: Pin<&mut SendFunction>, pChunk: &NetChunk);
    }

    extern "Rust" {
        #[cxx_name = "CHacks"]
        type Hacks;
        #[rust_name = "create_hacks"]
        fn CreateHacks(send: UniquePtr<SendFunction>) -> Box<Hacks>;
        #[cxx_name = "OnConnect"]
        fn on_connect(&mut self, client_id: i32);
        #[cxx_name = "OnDisconnect"]
        fn on_disconnect(&mut self, client_id: i32);
        #[cxx_name = "GetRecvPacket"]
        unsafe fn get_recv_packet<'a>(&'a mut self, packet: &mut NetChunk<'a>) -> bool;
        #[cxx_name = "OnRecvPacket"]
        fn on_recv_packet(&mut self, packet: &NetChunk) -> bool;
        #[cxx_name = "OnSendPacket"]
        fn on_send_packet(&mut self, packet: &NetChunk) -> bool;
        #[cxx_name = "OnSnap"]
        fn on_snap(&mut self, client_id: i32, snap_buffer: &mut [i32], snap_size: &mut usize);
        //#[cxx_name = "CreateDeltaServer"]
        //fn create_delta_server(&mut self, client_id: i32, from: &[i32], to: &[i32], delta: &mut [i32]) -> usize;
    }
}

pub struct Hacks {
    send: UniquePtr<SendFunction>,
    send_buffer: VecDeque<BufferedPacket>,
    // Always has length at least 1. The first packet is not considered to be
    // part of this collection. This allows us to return pointers to this first
    // element.
    recv_buffer: VecDeque<BufferedPacket>,
    proxies: Vec<Option<Box<dyn Proxy>>>,
}

struct BufferedPacket {
    client_id: i32,
    flags: i32,
    data: ArrayVec<[u8; 2048]>,
}

struct SendCallback<'a> {
    client_id: i32,
    buffer: &'a mut VecDeque<BufferedPacket>,
}

impl<'a> ddnet_backcompat_proxy::Send for SendCallback<'a> {
    fn send(&mut self, unreliable: bool) -> &mut ArrayVec<[u8; 2048]> {
        self.buffer.push_back(BufferedPacket {
            client_id: self.client_id,
            flags: if unreliable { 0 } else { NETSENDFLAG_VITAL },
            data: ArrayVec::new(),
        });
        &mut self.buffer.back_mut().unwrap().data
    }
}

pub fn create_hacks(send: UniquePtr<SendFunction>) -> Box<Hacks> {
    Box::new(Hacks {
        send,
        send_buffer: Default::default(),
        recv_buffer: [
            BufferedPacket { client_id: -1, flags: -1, data: ArrayVec::new() }
        ].into_iter().collect(),
        proxies: Default::default(),
    })
}

impl Hacks {
    fn flush_send_buffer(&mut self) {
        for &BufferedPacket { client_id, flags, ref data } in &self.send_buffer {
            self.send.pin_mut().send(&NetChunk {
                client_id,
                flags,
                data,
            });
        }
        self.send_buffer.clear();
    }
    pub fn on_connect(&mut self, client_id: i32) {
        let client_id: usize = client_id.try_into().unwrap();
        if client_id <= self.proxies.len() {
            self.proxies.resize_with(client_id + 1, || None);
        }
        self.proxies[client_id] = ddnet_backcompat_proxy::create_ddnet(123); // TODO
    }
    pub fn on_disconnect(&mut self, client_id: i32) {
        if let Some(proxy) = self.proxies.get_mut(usize::try_from(client_id).unwrap()) {
            *proxy = None;
        }
    }
    pub fn get_recv_packet<'a>(&'a mut self, packet: &mut NetChunk<'a>) -> bool {
        self.flush_send_buffer();
        if self.recv_buffer.len() < 2 {
            return false;
        }
        assert!(self.recv_buffer.pop_front().is_some());
        let &BufferedPacket { client_id, flags, ref data } = self.recv_buffer.front().unwrap();
        *packet = NetChunk {
            client_id,
            flags,
            data,
        };
        true
    }
    pub fn on_recv_packet(&mut self, packet: &NetChunk) -> bool {
        self.flush_send_buffer();
        if packet.client_id < 0 {
            return true;
        }
        if let Some(Some(proxy)) = self.proxies.get_mut(usize::try_from(packet.client_id).unwrap()) {
            let NetChunk { client_id, flags, data } = *packet;
            let result = proxy.translate_client_packet(
                data,
                if flags & NETSENDFLAG_VITAL == 0 { true } else { false },
                &mut SendCallback { client_id, buffer: &mut self.send_buffer },
                &mut SendCallback { client_id, buffer: &mut self.recv_buffer },
            );
            self.flush_send_buffer();
            result
        } else {
            true
        }
    }
    pub fn on_send_packet(&mut self, packet: &NetChunk) -> bool {
        self.flush_send_buffer();
        if packet.client_id < 0 {
            return true;
        }
        if let Some(Some(proxy)) = self.proxies.get_mut(usize::try_from(packet.client_id).unwrap()) {
            let NetChunk { client_id, flags, data } = *packet;
            let result = proxy.translate_server_packet(
                data,
                if flags & NETSENDFLAG_VITAL == 0 { true } else { false },
                &mut SendCallback { client_id, buffer: &mut self.recv_buffer },
                &mut SendCallback { client_id, buffer: &mut self.send_buffer },
            );
            if !result {
                self.flush_send_buffer();
            } else {
                // Don't flush the send buffer here because we first want the
                // current packet to get sent.
            }
            result
        } else {
            true
        }
    }
    pub fn on_snap(&mut self, client_id: i32, snap_buffer: &mut [i32], snap_size: &mut usize) {
        let _ = (client_id, snap_buffer, snap_size);
    }
    //pub fn create_delta_server(&mut self, PeerId: i32, pFrom: &[i32], pTo: &[i32], pDelta: &mut [i32]) -> usize {
    //}
}
