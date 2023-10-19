#[cfg(any(
    target_os = "linux",
    target_os = "windows",
    target_os = "macos"
))]
use clipboard::{ClipboardContext, ClipboardProvider};
use libp2p::swarm::dial_opts::DialOpts;

#[cfg(any(target_os = "android", target_os = "ios"))]
use crate::mobile::MobileClipboard;
#[cfg(any(target_os = "android", target_os = "ios"))]
use crate::utils::send_udp_msg_flutter;

use futures::{future::Either, StreamExt};
use libp2p::core::transport;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::{self, Kademlia};
use libp2p::Multiaddr;
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::OrTransport, upgrade},
    gossipsub,
    identity::Keypair,
    mdns, noise, quic,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, PeerId, Transport,
};
use std::collections::HashSet;
use std::error::Error;
use std::str::FromStr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::select;
use tokio::sync::mpsc::{self, Sender};
use tokio::time::{sleep, Duration};

use crate::utils::{
    get_keys, init_backend_listener, load_known_peers, save_peer,
};

#[macro_export]
macro_rules! desktop {
    // NOTE
    // use desktop!({}); if content can be in a separate block
    // and desktop! {}; if content needs to be in the same block as parent
    // In second case semicolon should be missed for last statement
    // If you will find way how to fix it -> feel free to open pr
    // same rules for mobile!
    // For all platforms
    ($($statement:stmt);*) => {
        $(
            #[cfg(any(
                target_os = "linux",
                target_os = "windows",
                target_os = "macos"
            ))]
            $statement;
        )*
    };
    // For selected specific only
    ($target_os:literal, $($statement:stmt);*) => {
        $(
            #[cfg(target_os = $target_os)]
            $statement;
        )*
    };
}
#[macro_export]
macro_rules! mobile {
    ($($statement:stmt);*) => {
        $(
            #[cfg(any(
                target_os = "android",
                target_os = "ios"
            ))]
            $statement;
        )*
    };
    ($target_os:literal, $($statement:stmt);*) => {
        $(
            #[cfg(target_os = $target_os)]
            $statement;
        )*
    };
}

pub async fn run_node(
    flutter_udp_port: Option<i32>,
) -> Result<(), Box<dyn Error>> {
    let mut app_dir_path: Option<String> = None;
    desktop!({
        pretty_env_logger::init();
        app_dir_path = None;
    });
    mobile!({
        app_dir_path = Some(
            send_udp_msg_flutter(
                "data_dir:".to_string(),
                &flutter_udp_port.unwrap(),
            )
            .await?,
        );
    });

    // Init keys
    let (local_key, local_peer_id) =
        get_keys(app_dir_path.clone(), flutter_udp_port).await?;
    println!("Local peer id: {}", &local_peer_id.to_string());
    log::info!("Local peer id: {}", &local_peer_id.to_string());

    // Listener for sending data to client apps
    desktop! {
        let backend_listener = init_backend_listener().await?
    }
    mobile! {
        let backend_listener = init_backend_listener(app_dir_path.unwrap()).await?
    }

    // Clipboard management
    desktop! {
        let mut clipboard: ClipboardContext =
            ClipboardProvider::new().expect("Failed to init clipboard")
    };
    mobile! {
        let mut clipboard: MobileClipboard =
            MobileClipboard::new(flutter_udp_port.clone().unwrap())
                .expect("Failed to init clipboard")

    };

    // Communications betwen threads
    // let (sender, mut receiver) = mpsc::channel(8192);

    // Store active peers
    let mut peers_online: Vec<(String, String)> = vec![];
    let mut peers_online_system: Vec<(PeerId, Multiaddr)> = vec![];

    let transport = build_transport(&local_key).await?;

    // Topic used to send clipboard updates
    let update_topic = gossipsub::IdentTopic::new("resk-update");

    // Build swarm
    let mut swarm = {
        // mdns config
        // Custom config to track active peers
        let mut mdns_config = mdns::Config::default();
        mdns_config.ttl = Duration::from_secs(60);
        let mdns = mdns::tokio::Behaviour::new(mdns_config, local_peer_id)?;

        // gossipsub config
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()?;
        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key),
            gossipsub_config,
        )?;

        // Subscribe to topics
        gossipsub.subscribe(&update_topic)?;

        // kademlia config
        let store = MemoryStore::new(local_peer_id);
        let kademlia = Kademlia::new(local_peer_id, store);
        let behaviour = Behaviour {
            mdns,
            gossipsub,
            kademlia,
        };
        SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id)
            .build()
    };

    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Pooling to share clipboard
    /*     tokio::spawn(start_pooling_clipboard(
        sender.clone(),
        flutter_udp_port.clone(),
    )); */
    // Load known peers
    let known_peers: Vec<PeerId> = load_known_peers()?;

    // Main pool
    // cloning swarm to multiple threads can create a mess
    let mut buf: Vec<u8> = Vec::new();
    loop {
        buf.clear();
        select! {
                    // Listen for requests from client apps
                    request = backend_listener.recv_buf_from(&mut buf) => match request {
                        Ok((_, addr)) => {
                            let request = String::from_utf8_lossy(&buf).to_string();
                            let mut response = String::new();
                            match request.split(":").nth(0).unwrap() {
                                    "is_alive" => {
                                        response = "1".to_string();
                                    }
                                    "get_peers" => {
                                        peers_online.clone().into_iter().for_each(|(peer_id, addr)| {
                                            response.push_str(format!("{peer_id}:{addr:?},").as_str())
                                        });
                                    }
                                    "add_peer" => {
                                        let peer_id = request.split(":").nth(1).unwrap();
                                        let peer_id: PeerId = peers_online_system
                                            .clone()
                                            .into_iter()
                                            .filter(|chunk| chunk.0.to_string() == peer_id)
                                            .nth(0).unwrap().0;
                                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                        save_peer(&peer_id).unwrap_or_else(|err| {log::error!("{err}")});
                                        response.push_str("OK");
                                    }
                                    "local_peer_id" => {
                                        response.push_str(&local_peer_id.to_string());
                                    }
                                    "get_peer_os" => {
                                        let peer_id = request.split(":").nth(1).unwrap();
                                        let key = kad::record::Key::new(&format!("{}_OS", peer_id));
                                        swarm.behaviour_mut().kademlia.get_record(key);
                                    }
                                    _ => {
                                        response.push_str("Incorrect request");},

                                    }
                            backend_listener.send_to(response.as_bytes(), addr).await.expect("Failed to ping back from server");
                        }
                        Err(e) => {
                            log::error!("Error obtaining request from client app: {e:?}");
                        }
                    },
                    // Listen for swarm events
                    event = swarm.select_next_some() => match event {
                        SwarmEvent::NewListenAddr { address, .. } => log::info!("Listening on {address:?}"),
                        SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(peers_list))) => {
                            peers_list.clone().into_iter().for_each(|(peer_id, addr)| {
                                // Store OS of node in network
                                // STORE ONLY PUBLIC DATA IN KAD
                                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                let key =
                                    kad::record::Key::new(&format!("{}_OS", local_peer_id.to_string()));
                                let record =
                                    kad::Record::new(key.clone(), std::env::consts::OS.as_bytes().to_vec());
                                swarm
                                    .behaviour_mut()
                                    .kademlia
                                    .put_record(record, kad::Quorum::All).expect("Failed to put record");
                                swarm.behaviour_mut().kademlia.start_providing(key.clone()).expect("Failed to start providing");
                            });

                            // Store active peers
                            if !peers_list.is_empty(){
                                peers_online_system.extend(peers_list.clone());
                            }
                            let peers_list = filter_incoming_peers(&peers_online, peers_list);
                            if !peers_list.is_empty() {
                                if !known_peers.is_empty() {
                                    let peer_id = PeerId::from_str(&peers_list.clone().into_iter().nth(0).unwrap().0)?;
                                    if known_peers.contains(&peer_id) {
                                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                    }
                                }
                                peers_online.extend(peers_list);
                            }
                        }
                        SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(peers_list))) => {
                            // Remove expired peers: it will be every 3 sec
                            peers_online_system.retain(|chunk| !peers_list.contains(&chunk));
                            let peers_list: Vec<(String, String)> = peers_list
                                .into_iter()
                                .map(|(peer_id, addr)| {
                                    (
                                        peer_id.to_string(),
                                        addr.to_string().split('/').nth(2).unwrap().to_string(),
                                    )
                                })
                                .collect();
                            peers_online.retain(|chunk| !peers_list.contains(&chunk));
                        }
                        SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message { propagation_source: _peer_id, message_id: _id, message })) => {
                            let msg_str = String::from_utf8_lossy(&message.data);
                            if message.topic.clone() == update_topic.hash() {
                                clipboard.set_contents(msg_str.to_string())?;
                            }
                        }
                        SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad::KademliaEvent::OutboundQueryProgressed {result, ..})) => {
                            match result {
                                kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders { key, providers, .. })) => {
                        for peer in providers {
                            println!(
                                "Peer {peer:?} provides key {:?}",
                                std::str::from_utf8(key.as_ref()).unwrap()
                            );
                        }
                    }
                    kad::QueryResult::GetProviders(Err(err)) => {
                        eprintln!("Failed to get providers: {err:?}");
                    }
                    kad::QueryResult::GetRecord(Ok(
                        kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                            record: kad::Record { key, value, .. },
                            ..
                        })
                    )) => {
                        println!(
                            "Got record {:?} {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap(),
                            std::str::from_utf8(&value).unwrap(),
                        );
                    }
                    kad::QueryResult::GetRecord(Ok(_)) => {}
                    kad::QueryResult::GetRecord(Err(err)) => {
                        eprintln!("Failed to get record: {err:?}");
                    }
                    kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                        println!(
                            "Successfully put record {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap()
                        );
                    }
                    kad::QueryResult::PutRecord(Err(err)) => {
                        eprintln!("Failed to put record: {err:?}");
                    }
                    kad::QueryResult::StartProviding(Ok(kad::AddProviderOk { key })) => {
                        println!(
                            "Successfully put provider record {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap()
                        );
                    }
                    kad::QueryResult::StartProviding(Err(err)) => {
                        eprintln!("Failed to put provider record: {err:?}");
                    }
                    _ => {}
                            }
                        }
                        _ => {}
                    },
                    // Listen for clipboard updates from pooler
        /*             update = receiver.recv() => match update {
                        Some(updated_clipboard) => {
                            if swarm.behaviour_mut().gossipsub.all_peers().count() > 0 {
                               swarm.behaviour_mut().gossipsub.publish(update_topic.clone(), updated_clipboard.as_bytes())?;
                               log::info!("Shared clipboar content with peers");
                            }
                        }
                        _ => {}
                    } */
                }
    }
}

async fn start_pooling_clipboard(
    sender: Sender<String>,
    flutter_udp_port: Option<i32>,
) {
    // Init clipboard
    desktop! {
        let mut clipboard: ClipboardContext =
            ClipboardProvider::new().expect("Failed to init clipboard")
    };
    mobile! {
        let mut clipboard: MobileClipboard =
            MobileClipboard::new(flutter_udp_port.clone().unwrap())
                .expect("Failed to init clipboard")

    };

    // Get initial clipboard content
    let mut last_clipboard_content = clipboard
        .get_contents()
        .expect("Failed to get clipboard content");

    // Check if it was changed
    loop {
        sleep(Duration::from_secs(1)).await;
        let current_clipboard_content = clipboard
            .get_contents()
            .expect("Failed to get clipboard content");
        if current_clipboard_content != last_clipboard_content {
            last_clipboard_content = current_clipboard_content.clone();
            sender
                .send(last_clipboard_content.clone())
                .await
                .expect("Failed to send value");
        }
    }
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    mdns: mdns::tokio::Behaviour,
    gossipsub: gossipsub::Behaviour,
    kademlia: Kademlia<MemoryStore>,
}

async fn build_transport(
    local_key: &Keypair,
) -> Result<transport::Boxed<(PeerId, StreamMuxerBox)>, Box<dyn Error>> {
    let tcp_transport =
        tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
            .upgrade(upgrade::Version::V1Lazy)
            .authenticate(noise::Config::new(local_key)?)
            .multiplex(yamux::Config::default())
            .timeout(Duration::from_secs(20))
            .boxed();
    let quic_transport =
        quic::tokio::Transport::new(quic::Config::new(local_key));

    let transport = OrTransport::new(quic_transport, tcp_transport)
        .map(|either_output, _| match either_output {
            Either::Left((peer_id, muxer)) => {
                (peer_id, StreamMuxerBox::new(muxer))
            }
            Either::Right((peer_id, muxer)) => {
                (peer_id, StreamMuxerBox::new(muxer))
            }
        })
        .boxed();
    Ok(transport)
}

fn filter_incoming_peers(
    peers_online_list: &Vec<(String, String)>,
    peers_list: Vec<(PeerId, Multiaddr)>,
) -> Vec<(String, String)> {
    let mut response: Vec<(String, String)> = vec![];
    // Remove duplicates
    let mut peers_ids: HashSet<String> = HashSet::new();
    for (peer_id, addr) in peers_list {
        if peers_ids.insert(peer_id.to_string()) {
            response.push((
                peer_id.to_string(),
                addr.to_string().split('/').nth(2).unwrap().to_string(),
            ));
        }
    }

    let existing_peers_ids: HashSet<String> = peers_online_list
        .iter()
        .map(|(peer_id, _)| peer_id.to_string().clone())
        .collect();
    let response: Vec<(String, String)> = response
        .into_iter()
        .filter(|(peer_id, _)| !existing_peers_ids.contains(peer_id))
        .collect();
    response
}
