
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use futures_channel::mpsc::UnboundedSender;
use std::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use crate::sdp::*;

pub type Tx = UnboundedSender<Message>;
pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;
pub type DevicesMap = Arc<Mutex<Devices>>;



pub struct Device {
    pub addr: SocketAddr,
    pub offer: Option<SessionDescription>,
    pub answer: Option<SessionDescription>,
}

pub struct Devices {
    pub list: HashMap<String, Device>,
}


impl Device {
    pub fn new(addr: SocketAddr) -> Self {
        Device {
            addr,
            offer: None,
            answer: None,
        }
    }

    pub fn recv_offer(&mut self, offer: SessionDescription) {
        self.offer = Some(offer);
    }

    pub fn recv_answer(&mut self, answer: SessionDescription) {
        self.answer = Some(answer);
    }

    pub fn address(&self) -> SocketAddr {
        self.addr
    }
}

impl Devices {
    pub fn new() -> Self {
        Devices {
            list: HashMap::new(),
        }
    }

    pub fn add_device(&mut self, device_id: String, addr: SocketAddr) {
        let device = Device::new(addr);
        self.list.insert(device_id, device);
    }

    pub fn remove_device(&mut self, device_id: &String) {
        self.list.remove(device_id);
    }

    pub fn get_name(&self, addr: &SocketAddr) -> Option<String> {
        for (name, device) in &self.list {
            if &device.address() == addr {
                return Some(name.clone());
            }
        }
        None
    }
}