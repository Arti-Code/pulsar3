extern crate colored;
use std::io::Write;
use std::env;
use std::time::Duration;
use futures_util::{
    future,
    pin_mut, 
    StreamExt
};
//use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::io::*;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::{Utf8Bytes, protocol::Message}};
use colored::Colorize;
use pulsar::prelude::*;
type ChatSender = futures_channel::mpsc::UnboundedSender<Message>;


#[tokio::main]
async fn main() {
    init_display().await;
    let user_name= register_name().await;
    let user_name: &'static str  = Box::leak(user_name.into_boxed_str());
    let url = get_connection_address().await;
    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(send_message(stdin_tx.clone(), &user_name));
    let ws_stream = establish_connection(&url).await;
    send_command(stdin_tx.clone(), &DeviceMessage::Register(RegisterPeer {
        name: user_name.to_string(),
    })).await;
    let (write, read) = ws_stream.split();
    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            match message {
                Err(e) => {
                    println!("Error receiving message: {}", e);
                    return;
                }
                Ok(msg) => recv_messages(msg).await,
            }
            
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

async fn establish_connection(url: &str) -> WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>> {
    print!("connecting to: {}...", url);
    _ = std::io::stdout().flush().expect("ups!");
    tokio::time::sleep(Duration::from_secs(1)).await;
    let (ws_stream, _) = connect_async(url).await.expect("failed to connect");
    println!("[OK]");
    println!("---------------------");
    ws_stream
}

async fn recv_messages(msg: Message) {
    let text = msg.to_text().expect("unreadable data");
    match serde_json::from_str::<DeviceMessage>(&text) {
        Ok(DeviceMessage::Offer(sdp)) => {
            println!("offer: {} => {}", sdp.sender(), sdp.description());
        },
        Ok(DeviceMessage::ListUsersResponse(data)) => {
            let text = data.split_terminator('\n');
            println!("{}", "--- Connected Users ---".to_string().blue().underline());
            for user in text {
                println!("{}", user.bright_cyan());
            }
            println!("-----------------------");
        },
        Ok(DeviceMessage::Answer(sdp)) => {
            println!("answer: {} => {}", sdp.sender(), sdp.description());
        },
        Ok(_) => {
            println!("other cmd: {}", text);
        },
        Err(e) => {
            eprintln!("{}", e);
            dbg!(&text);
            //println!("{}", text);
        }
    }
}

async fn send_message(tx: ChatSender, user_name: &str) {
    let user = format!("[{}] ", user_name).as_bytes().to_vec();
    loop {
        let mut stdin = tokio::io::stdin();
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        
        buf.truncate(n);
        match check_command(std::str::from_utf8(&buf).unwrap(), user_name).await {
            Some(command) => {
                send_command(tx.clone(), &command).await;
                if let DeviceMessage::Quit = command { break; }
                continue;
            }
            None => {
                let mut msg = user.clone(); 
                msg.extend_from_slice(&buf);
                unsafe {
                    let text = Utf8Bytes::from_bytes_unchecked(msg.into());
                    if tx.unbounded_send(Message::Text(text)).is_err() {
                        break;
                    }
                }
            }
        }        
    }
}

async fn check_command(input: &str, sender: &str) -> Option<DeviceMessage> {
    let command = input.trim().to_string();
    let cmd = command.split_whitespace().collect::<Vec<&str>>();
    if let Some(c1) = cmd.first() {
        return match *c1 {
            "quit" => Some(DeviceMessage::Quit),
            "list" => Some(DeviceMessage::ListUsersRequest),
            "offer" => {
                let receiver = cmd.get(1).unwrap().to_string();
                let sdp = SessionDescription::random(receiver, sender.to_string(), SdpType::Offer);
                Some(DeviceMessage::Offer(sdp))
            },
            "answer" => {
                let receiver = cmd.get(1).unwrap().to_string();
                let sdp = SessionDescription::random(receiver, sender.to_string(), SdpType::Answer);
                Some(DeviceMessage::Answer(sdp))
            },
            _ => None,
        }
    } else {
        None
    }
}

async fn send_command(tx: ChatSender, command: &DeviceMessage) -> bool {
    let cmd_msg = serde_json::to_string(command).unwrap();
    let _ = tx.unbounded_send(Message::Text(cmd_msg.into()));
    return false;
}

async fn register_name() -> String {
    print!("Enter your user name: ");
    std::io::stdout().flush().expect("ups! something went wrong!");
    let mut buf = String::new();
    _ = std::io::stdin().read_line(&mut buf).expect("failed to register user name");
    let name = buf.trim();
    println!("Welcome, {}!", name.green());
    name.to_string()
}

async fn init_display() {
    let ver = env!("CARGO_PKG_VERSION");
    let author = env!("CARGO_PKG_AUTHORS");
    println!(" ");
    let title = "-=PULSAR Client=-".to_string().green().bold().underline();
    println!(" ");
    println!("{}", title);
    println!("ver. {}", ver);
    println!("{}", author);
    println!(" ");
    tokio::time::sleep(Duration::from_secs(1)).await;
}

async fn get_connection_address() -> String {
    match env::args().nth(1) {
        Some(url) => url,
        //None => "ws://127.0.0.1:8080".to_string(),
        None => "ws://crossover.proxy.rlwy.net:15492".to_string(),
    }
}
