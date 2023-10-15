use libp2p::{identity::Keypair, PeerId};
use serde_json::{Map, Value};
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::BufWriter,
    path::{Path, PathBuf},
};
#[cfg(any(
    target_os = "linux",
    target_os = "windows",
    target_os = "macos"
))]
use {
    lazy_static::lazy_static,
    std::{env, str::FromStr},
};

use tokio::net::UdpSocket;

use crate::{desktop, mobile};

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
lazy_static! {
    // Reference to application's directory
    pub static ref APP_DIR: PathBuf = {
        let home = env::var("HOME").expect("HOME environment variable not found");
        Path::new(&home).join(".resk")
    };
    // Reference to common data
    pub static ref DATA_MAP: PathBuf = APP_DIR.clone().join("data.json");
}

pub async fn get_keys(
    data_dir: Option<String>,
    flutter_udp_port: Option<i32>,
) -> Result<(Keypair, PeerId), Box<dyn Error>> {
    // Declaration
    let shared_dir_path: String;
    let data_dir_path: PathBuf;

    // Initialization
    desktop!({
        shared_dir_path = env::var("HOME")?;
        data_dir_path = APP_DIR.clone();
    });
    mobile!({
        shared_dir_path = send_udp_msg_flutter(
            "root_dir:".to_string(),
            &flutter_udp_port.unwrap(),
        )
        .await?;
        data_dir_path =
            Path::new(&data_dir.unwrap().trim_matches('\0')).to_path_buf();
    });
    let shared_dir_path =
        Path::new(&shared_dir_path.trim_matches('\0')).join("Resk");

    // Create dirs
    fs::create_dir_all(&data_dir_path)?;
    fs::create_dir_all(&shared_dir_path)?;

    // Wrap paths
    let data_map_path = data_dir_path.clone().join("data.json");
    let peer_key_path = data_dir_path.clone().join("peer_key.dat");
    let mut data_map = load_data_map(&data_map_path)?;

    let local_key: Keypair;
    if !Path::exists(&peer_key_path) {
        // First generate
        local_key = Keypair::generate_ed25519();

        // Then save private key
        let local_key_bytes = local_key.to_protobuf_encoding()?;
        fs::write(&peer_key_path, local_key_bytes)?;

        // Then save public key
        data_map.insert(
            "local_peer_id".to_string(),
            Value::String(local_key.public().to_peer_id().to_string()),
        );
        write_json(&data_map_path, &data_map)?;

        log::info!("Saving peer key to {peer_key_path:?} and data.json")
    } else {
        let local_key_bytes = fs::read(&peer_key_path)?;
        local_key = Keypair::from_protobuf_encoding(&local_key_bytes)?;
        log::info!("Using peer key from {peer_key_path:?}")
    }

    let local_peer_id = PeerId::from(local_key.public());

    Ok((local_key, local_peer_id))
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub async fn save_port(port: u16) -> Result<(), Box<dyn Error>> {
    let mut data_map = load_data_map(&*DATA_MAP)?;

    data_map.insert("port".to_string(), Value::String(port.to_string()));

    write_json(&*DATA_MAP, &data_map)?;
    Ok(())
}
#[cfg(any(target_os = "android", target_os = "ios"))]
pub async fn save_port(
    port: u16,
    data_dir: String,
) -> Result<(), Box<dyn Error>> {
    let data_map_path = Path::new(&data_dir).join("data.json").to_path_buf();
    let mut data_map = load_data_map(&data_map_path)?;

    data_map.insert("port".to_string(), Value::String(port.to_string()));

    write_json(&data_map_path, &data_map)?;
    Ok(())
}

pub fn load_data_map(
    path: &PathBuf,
) -> Result<Map<String, Value>, Box<dyn Error>> {
    let data: Map<String, Value>;
    if !path.exists() {
        data = Map::new();
        let _ = OpenOptions::new().write(true).create(true).open(path)?;
    } else {
        let json_str = fs::read_to_string(&path)?;
        data = serde_json::from_str(&json_str)?;
    }
    Ok(data)
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub fn save_peer(peer_id: &PeerId) -> Result<(), Box<dyn Error>> {
    // Define vars
    let mut data_map = load_data_map(&DATA_MAP)?;
    let mut peers_list: Vec<Value>;

    // Check if peers already saved
    if data_map.contains_key("peers") {
        peers_list = data_map.get("peers").unwrap().as_array().unwrap().clone();
    } else {
        peers_list = Vec::<Value>::new();
    }
    // Edit array
    peers_list.push(Value::String(peer_id.to_string()));
    data_map.insert("peers".to_string(), Value::Array(peers_list));

    // Write back
    write_json(&*DATA_MAP, &data_map)?;
    Ok(())
}
#[cfg(any(target_os = "android", target_os = "ios"))]
pub fn save_peer(_peer_id: &PeerId) -> Result<(), Box<dyn Error>> {
    Ok(())
}

fn write_json(
    path: &PathBuf,
    data: &Map<String, Value>,
) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, data)?;
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub fn load_known_peers() -> Result<Vec<PeerId>, Box<dyn Error>> {
    let mut known_peers = vec![];
    let data_map = load_data_map(&DATA_MAP)?;
    if data_map.contains_key("peers") {
        let peers_list =
            data_map.get("peers").unwrap().as_array().unwrap().clone();
        for peer_id in peers_list.into_iter() {
            let peer_id = PeerId::from_str(&peer_id.as_str().unwrap())?;
            known_peers.push(peer_id);
        }
    }
    Ok(known_peers)
}
#[cfg(any(target_os = "android", target_os = "ios"))]
pub fn load_known_peers() -> Result<Vec<PeerId>, Box<dyn Error>> {
    Ok(vec![])
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub async fn init_backend_listener() -> Result<UdpSocket, Box<dyn Error>> {
    let backend_listener = UdpSocket::bind("127.0.0.1:0").await?;
    let port = backend_listener.local_addr().unwrap().port();
    save_port(port).await?;
    log::info!("Waiting for messages from client apps on {backend_listener:?}");
    Ok(backend_listener)
}
#[cfg(any(target_os = "android", target_os = "ios"))]
pub async fn init_backend_listener(
    data_dir: String,
) -> Result<UdpSocket, Box<dyn Error>> {
    let backend_listener = UdpSocket::bind("127.0.0.1:0").await?;
    let port = backend_listener.local_addr().unwrap().port();
    save_port(port, data_dir).await?;
    log::info!("Waiting for messages from client apps on {backend_listener:?}");
    Ok(backend_listener)
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub async fn send_udp_msg_flutter(
    request: String,
    port: &i32,
) -> Result<String, Box<dyn Error>> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    let server_addr = format!("127.0.0.1:{port}");
    socket.send_to(request.as_bytes(), server_addr).await?;
    let mut buf = Vec::<u8>::new();
    socket.recv_buf(&mut buf).await?;
    let buf = String::from_utf8_lossy(&buf);
    Ok(buf.to_string())
}
