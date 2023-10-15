use lazy_static::lazy_static;
use serde_json::{Map, Value};
use std::path::{self, Path, PathBuf};
use std::{env, error::Error};
use tokio::net::UdpSocket;

use clap::{command, Arg, Command};

lazy_static! {
    // Reference to application's directory
    pub static ref APP_DIR: path::PathBuf = {
        let home = env::var("HOME").expect("HOME environment variable not found");
        Path::new(&home).join(".resk")
    };
    pub static ref BACKEND_ADDR: String = {
        let addr = format!("127.0.0.1:{}", get_port().expect("Check if resk_node is running"));
        addr
    };
}

pub async fn run() -> Result<(), Box<dyn Error>> {
    // First check
    let alive_ping = send_msg("is_alive:".to_string());
    let matches = command!()
        .subcommand(
            Command::new("get_peers").about("Get peers in local network"),
        )
        .subcommand(
            Command::new("add_peer")
                .about("Add peer to resk network")
                .arg(Arg::new("peer_id").required(true)),
        )
        .subcommand(Command::new("local").about("Get local peer id"))
        .subcommand_required(true)
        .get_matches();
    alive_ping.await?;
    if let Some(_matches) = matches.subcommand_matches("get_peers") {
        println!("block1");
        get_peers().await?;
    }
    if let Some(matches) = matches.subcommand_matches("add_peer") {
        let peer_id = matches.get_one::<String>("peer_id").unwrap();
        add_peer(peer_id).await?;
    }
    if let Some(_matches) = matches.subcommand_matches("local") {
        get_local_peer_id().await?
    }
    Ok(())
}

fn get_port() -> Result<String, Box<dyn Error>> {
    let data_path = APP_DIR.clone().join("data.json");
    let data_map = load_data_map(&data_path);
    let data_map = match data_map {
        Ok(data_map) => data_map,
        Err(_) => return Err("Check if resk_node is running".into()),
    };
    let port = data_map.get("port");
    let port = match port {
        Some(port) => port.as_str().to_owned().unwrap().to_string(),
        None => return Err("Check if resk_node is running".into()),
    };
    Ok(port)
}

async fn send_msg(request: String) -> Result<String, Box<dyn Error>> {
    let mut buf: Vec<u8> = Vec::new();
    let sender = UdpSocket::bind("127.0.0.1:0").await?;
    sender
        .send_to(request.as_bytes(), BACKEND_ADDR.clone())
        .await
        .expect("Failed to send");
    sender
        .recv_buf(&mut buf)
        .await
        .expect("Failed to recv back");
    Ok(String::from_utf8_lossy(&buf).to_string())
}

async fn get_peers() -> Result<(), Box<dyn Error>> {
    let peers = send_msg("get_peers:".to_string()).await?;

    if !peers.is_empty() {
        println!("Peers in local network:");
        for peer_record in peers.split(",").into_iter() {
            if !peer_record.is_empty() {
                let peer_id =
                    peer_record.split(":").nth(0).expect("Incorrect response");
                let address =
                    peer_record.split(":").nth(1).expect("Incorrect response");
                println!("{} with address {}", peer_id, address)
            }
        }
    } else {
        println!("No peers in local network found")
    }
    Ok(())
}

async fn add_peer(peer_id: &String) -> Result<(), Box<dyn Error>> {
    let response = send_msg(format!("add_peer:{}", peer_id)).await?;
    if response != "OK".to_string() {
        println!("Something happend: {}", response);
    } else {
        println!("Peer has been added successfully");
    }
    Ok(())
}

fn load_data_map(path: &PathBuf) -> Result<Map<String, Value>, Box<dyn Error>> {
    let data: Map<String, Value>;
    if !path.exists() {
        data = Map::new();
    } else {
        let json_str = std::fs::read_to_string(&path)?;
        data = serde_json::from_str(&json_str)?;
    }
    Ok(data)
}

async fn get_local_peer_id() -> Result<(), Box<dyn Error>> {
    let response = send_msg("local_peer_id:".to_string()).await?;
    println!("Local peer id is -> {response}");
    Ok(())
}
