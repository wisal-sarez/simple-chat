use async_std::io::BufReader;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::{Arc, Mutex};
use async_std::task;
use server::ClientMessage;
use std::collections::HashMap;
use std::io::Result;

type UserStreamExt = Arc<Mutex<HashMap<String, TcpStream>>>;

async fn handle_client(stream: TcpStream, user_clients: UserStreamExt) {
    let addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to get peer address: {}", e);
            return;
        }
    };

    let reader = BufReader::new(stream.clone());
    let mut lines = reader.lines();
    let mut current_username: Option<String> = None;

    while let Some(line_res) = lines.next().await {
        match line_res {
            Ok(line) => {
                match serde_json::from_str::<ClientMessage>(&line) {
                    Ok(msg) => match msg {
                        ClientMessage::Connect { username } => {
                            let mut clients = user_clients.lock().await;
                            if clients.contains_key(&username) {
                                eprintln!("Username {} already exists", username);
                                return;
                            }
                            clients.insert(username.clone(), stream.clone());
                            current_username = Some(username.clone());
                            println!("{} connected", username);
                        }
                        ClientMessage::SendMessage { username, message } => {
                            if let Some(curr) = &current_username {
                                if curr != &username {
                                    eprintln!("Auth error: {} tried to send as {}", curr, username);
                                    continue;
                                }
                            } else {
                                continue;
                            }

                            // Clone the map to avoid holding the lock during async writes
                            let clients = user_clients.lock().await.clone();
                            let broadcast_msg = format!("{}: {}\n", username, message);

                            for (client_username, mut client_stream) in clients {
                                if client_username != username {
                                    // Ignore errors if a client disconnects during broadcast
                                    let _ = client_stream.write_all(broadcast_msg.as_bytes()).await;
                                }
                            }
                        }
                        ClientMessage::Leave { username } => {
                            let mut clients = user_clients.lock().await;
                            clients.remove(&username);
                            println!("{} left", username);
                            return;
                        }
                    },
                    Err(e) => eprintln!("JSON error from {}: {}", addr, e),
                }
            }
            Err(e) => {
                eprintln!("Read error from {}: {}", addr, e);
                break;
            }
        }
    }

    if let Some(username) = current_username {
        let mut clients = user_clients.lock().await;
        clients.remove(&username);
        println!("{} disconnected", username);
    }
}

async fn run_server(addr: &str) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Server running on {}", addr);

    let user_clients: UserStreamExt = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, _) = listener.accept().await?;
        task::spawn(handle_client(stream, user_clients.clone()));
    }
}

fn main() -> Result<()> {
    task::block_on(run_server("127.0.0.1:8080"))
}