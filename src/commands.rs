use serde::{Deserialize, Serialize};
use crate::sdp::*;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DeviceMessage {
    Register(RegisterPeer),
    Offer(SessionDescription),
    Answer(SessionDescription),
    Quit,
    ListUsersRequest,
    ListUsersResponse(String),
    Registered(RegisterData),
    Echo(String),
    Unknown,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterPeer {
    pub name: String,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterData {
    pub name: String,
    pub addr: String,
}