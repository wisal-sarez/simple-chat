//! Simple Async Chat Server
//!
//! This module implements an asynchronous TCP chat server that manages
//! multiple concurrent client connections in a single chat room.
//!
//! # Architecture
//! - Uses `async-std` for asynchronous I/O operations
//! - Maintains shared state of connected users via `Arc<Mutex<HashMap>>`
//! - Spawns a new task for each client connection
//! - Broadcasts messages to all connected users (excluding sender)
//!
//! # Protocol
//! Clients communicate with the server using JSON-serialized `ClientMessage`
//! structures, with each message terminated by a newline character.
//!
//! # Concurrency
//! The server handles multiple clients concurrently without blocking,
//! making it suitable for high-throughput scenarios.

use async_std::io::BufReader;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::{Arc, Mutex};
use async_std::task;
use server::ClientMessage;
use std::collections::HashMap;
use std::io::Result;

/// Shared thread-safe container for tracking connected users.
///
/// This type alias represents a concurrent hash map that maps usernames
/// to their associated TCP streams. It's wrapped in:
/// - `Arc`: For shared ownership across tasks
/// - `Mutex`: For safe concurrent access
type UserStreamExt = Arc<Mutex<HashMap<String, TcpStream>>>;

/// Handles an individual client connection for its entire lifecycle.
///
/// This function runs in a spawned task and processes all messages from
/// a single client until they disconnect or send a leave message.
///
/// # Parameters
/// - `stream`: The TCP stream for communicating with this client
/// - `user_clients`: Shared reference to all connected users
///
/// # Behavior
/// 1. Reads JSON messages line by line from the client
/// 2. Processes `Connect`, `SendMessage`, and `Leave` messages
/// 3. Validates usernames and prevents duplicates
/// 4. Broadcasts messages to other users
/// 5. Cleans up resources on client disconnect
///
/// # Error Handling
/// - Logs errors but doesn't propagate them (individual client failures
///   shouldn't crash the server)
/// - Automatically removes users who disconnect unexpectedly
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

/// Runs the main server loop, accepting and handling client connections.
///
/// # Parameters
/// - `addr`: The socket address to bind to (e.g., "127.0.0.1:8080")
///
/// # Returns
/// - `Result<()>`: Returns `Ok(())` if the server runs successfully,
///   or an `io::Error` if binding to the address fails.
///
/// # Behavior
/// 1. Binds to the specified address
/// 2. Initializes shared state for user tracking
/// 3. Enters an infinite loop accepting connections
/// 4. Spawns a new task for each accepted connection
///
/// # Note
/// This function runs indefinitely until the process is terminated.
async fn run_server(addr: &str) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Server running on {}", addr);

    let user_clients: UserStreamExt = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, _) = listener.accept().await?;
        task::spawn(handle_client(stream, user_clients.clone()));
    }
}

/// Entry point for the chat server application.
///
/// # Returns
/// - `Result<()>`: Exit code for the process (0 for success, non-zero for error)
///
/// # Default Configuration
/// Currently binds to `127.0.0.1:8080` by default. This could be extended
/// to accept command-line arguments for configuration.
fn main() -> Result<()> {
    task::block_on(run_server("0.0.0.0:8000"))
}
