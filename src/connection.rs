
use std::net::SocketAddr;
use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use serde_json;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use crate::device::*;
use crate::{DevicesMap, PeerMap};
use crate::commands::*;



pub async fn handle_connection(peer_map: PeerMap, device_map: DevicesMap, raw_stream: TcpStream, addr: SocketAddr) {
    let ws_stream: tokio_tungstenite::WebSocketStream<TcpStream> = tokio_tungstenite::accept_async(raw_stream)
        .await.expect("error during the websocket handshake occurred");
    //println!("new connection: {}", addr);
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx);
    let (outgoing, incoming) = ws_stream.split();
    let broadcast_incoming = incoming.try_for_each(|msg| {
        match msg.to_text() {
            Ok(text) => process_message(text, &addr, &device_map, &peer_map),
            Err(e) => {
                println!("error processing message from {}: {}", addr, e);
                return future::ok(());
            }
        }
        future::ok(())
    });
    let receive_from_others = rx.map(Ok).forward(outgoing);
    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;
    //println!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
}

fn process_message(msg: &str, addr: &SocketAddr, device_map: &DevicesMap, peer_map: &PeerMap) {
    let msg = serde_json::from_str::<DeviceMessage>(msg);
    let sender_name = match device_map.lock().unwrap().get_name(addr) {
        Some(name) => name,
        None => "unnamed".to_string(),
    };
    let log;
    let response = match msg {
        Ok(DeviceMessage::Register(reg)) => {
            let mut devices = device_map.lock().unwrap();
            devices.list.insert(reg.name.clone(), Device::new(*addr));
            log = format!("[REGISTER]: {} ({})", reg.name, addr);
            let peer_data = serde_json::to_string(&DeviceMessage::Registered(RegisterData {
                name: reg.name.clone(),
                addr: addr.to_string(),
            })).unwrap();
            peer_data
        },
        Ok(DeviceMessage::Offer(sdp)) => {
            let mut devices = device_map.lock().unwrap();
            if let Some(device) = devices.list.get_mut(&sdp.receiver()) {
                device.recv_offer(sdp.clone());
                let peers = peer_map.lock().unwrap();
                peers.get(&device.address()).map(|ws_sink| {
                    let offer = serde_json::to_string(&DeviceMessage::Offer(sdp.clone())).unwrap();
                    ws_sink.unbounded_send(Message::text(offer)).unwrap();
                });
                log = format!("[OFFER]: ({}->{}): {}", sdp.sender(), sdp.receiver(), sdp.description());
                serde_json::to_string(&DeviceMessage::Offer(sdp)).unwrap()
            } else {
                log = format!("{} not found", sdp.receiver());
                log.clone()
            }
        },
        Ok(DeviceMessage::Answer(sdp)) => {
            let mut devices = device_map.lock().unwrap();
            if let Some(device) = devices.list.get_mut(&sdp.receiver()) {
                device.recv_answer(sdp.clone());
                let peers = peer_map.lock().unwrap();
                peers.get(&device.address()).map(|ws_sink| {
                    let answer = serde_json::to_string(&DeviceMessage::Answer(sdp.clone())).unwrap();
                    ws_sink.unbounded_send(Message::text(answer)).unwrap();
                });
                log = format!("[ANSWER]: ({}->{}): {}", sdp.sender(), sdp.receiver(), sdp.description());
                serde_json::to_string(&DeviceMessage::Answer(sdp)).unwrap()
            } else {
                log = format!("{} not found", sdp.receiver());
                log.clone()
            }
        },
        Ok(DeviceMessage::Quit) => {
            let mut devices = device_map.lock().unwrap();
            devices.list.retain(|_, device| device.addr != *addr);
            peer_map.lock().unwrap().remove(addr);
            log = format!("[OFFLINE]: {}", sender_name);
            log.clone()
        },
        Ok(DeviceMessage::ListUsersRequest) => {
            let mut resp = String::new();
            let devices = device_map.lock().unwrap();
            for (name, device) in devices.list.iter() {
                let offer = match device.offer {
                    None => "offer: --".to_string(),
                    Some(ref offer) => format!("offer: {}", offer.description),
                };
                let answer = match device.answer {
                    None => "answer: --".to_string(),
                    Some(ref answer) => format!("answer: {}", answer.description),
                };
                /* let d = match device.offer {
                    None => format!("[{}]: offer: --", *name),
                    Some(ref offer) => format!("[{}]: offer: {}", *name, offer.description()),
                }; */
                resp.push_str(name.to_uppercase().as_str());
                resp.push_str(": ");
                resp.push_str(&offer);
                resp.push_str(" | ");
                resp.push_str(&answer);
                resp.push('\n');
            }
            log = resp.clone();
            serde_json::to_string(&DeviceMessage::ListUsersResponse(resp)).unwrap()
        },
        Ok(DeviceMessage::ListUsersResponse(_)) => {
            log = format!("ListUsersResponse");
            log.clone()
        },
        Ok(DeviceMessage::Echo(text)) => {
            log = format!("[ECHO]: {}", text);
            serde_json::to_string(&DeviceMessage::Echo(text)).unwrap()
        },
        Ok(_) => {
            log = format!("Unknown command");
            serde_json::to_string(&DeviceMessage::Unknown).unwrap()
        },
        Err(_) => {
            log = format!("Failed to parse message");
            log.clone()
        },
    };
    println!("{}", &log);
    let peers = peer_map.lock().unwrap();
    peers.get(addr).map(|ws_sink| {
        ws_sink.unbounded_send(Message::text(response)).unwrap();
    });
}