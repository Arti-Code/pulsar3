pub mod connection;
pub mod device;
pub mod commands;
pub mod sdp;


use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::Mutex,
};
use anyhow::Error;
use tokio::net::TcpListener;
use colored::*;
use crate::{connection::handle_connection, device::{DevicesMap, PeerMap}};
use crate::device::Devices;


#[tokio::main]
async fn main() -> Result<(), IoError> {
    init_display().await;
    
    let addr = env::args().nth(1).unwrap_or_else(|| "0.0.0.0:8080".to_string());
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("failed to bind");
    match listening(&listener).await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{}", e);
            Err(IoError::last_os_error())
        }
    }
    
    
}

async fn init_display() {
    let ver = env!("CARGO_PKG_VERSION");
    let author = env!("CARGO_PKG_AUTHORS");
    let descr = env!("CARGO_PKG_DESCRIPTION");
    println!("");
    println!("{}", "-=PULSAR SIGNALING SERVER=-".bold().red().underline());
    println!("");
    println!("ver: {}", ver);
    println!("{}", author);
    println!("{}", descr.italic());
    println!("");
}

async fn listening(listener: &TcpListener) -> Result<(), Error> {
    let peer_map: PeerMap = PeerMap::new(Mutex::new(HashMap::new()));
    let device_map: DevicesMap = DevicesMap::new(Mutex::new(Devices::new()));  
    println!("listening on {}", listener.local_addr()?.to_string().bright_blue().underline());
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(peer_map.clone(), device_map.clone(), stream, addr));
    }
    Ok(())
}