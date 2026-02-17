use serde::{Deserialize, Serialize};
use rand::{distr::Alphanumeric, prelude::*, rng};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionDescription {
    pub sender: String,
    pub receiver: String,
    pub description: String,
    pub sdp_type: SdpType
}

impl SessionDescription {
    pub fn new(sender: String, receiver: String, description: String, sdp_type: SdpType) -> Self {
        SessionDescription {
            sender,
            receiver,
            description,
            sdp_type,
        }
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn sender(&self) -> String {
        self.sender.clone()
    }

    pub fn receiver(&self) -> String {
        self.receiver.clone()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn is_offer(&self) -> bool {
        matches!(self.sdp_type, SdpType::Offer)
    }

    pub fn is_answer(&self) -> bool {
        matches!(self.sdp_type, SdpType::Answer)
    }

    pub fn random(receiver: String, sender: String, sdp_type: SdpType) -> Self {
        let rng = rng();
        let data = rng.sample_iter(Alphanumeric).take(16).collect::<Vec<u8>>();
        let sdp = String::from_utf8(data).unwrap();
        Self {
            receiver,
            sender,
            description: sdp,
            sdp_type,
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SdpType {
    Offer,
    Answer,
}
