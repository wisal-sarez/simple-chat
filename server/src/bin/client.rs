//! Simple Async Chat Client
//!
//! This module implements an asynchronous command-line interface for the chat server.
//! It provides a user-friendly CLI for connecting to the chat room, sending messages,
//! and receiving messages from other users.
//!
//! # Usage
//! ```bash
//! # Using command-line arguments
//! client <username> [host] [port]
//!
//! # Using environment variables
//! export USERNAME=alice
//! export HOST=127.0.0.1
//! export PORT=8080
//! client
//!
//! # Interactive commands
//! > send Hello everyone!
//! > leave
//! ```
//!
//! # Protocol
//! Communicates with the server using JSON-serialized `ClientMessage` structures
//! over a TCP connection, with newline-delimited messages.

use async_std::io::{self, BufReader};
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task;
use server::ClientMessage;
use std::env;

/// Continuously listens for incoming messages from the server and displays them.
///
/// This function runs in a separate task to allow simultaneous message reception
/// and user input handling.
///
/// # Parameters
/// - `stream`: TCP stream connected to the chat server
///
/// # Behavior
/// - Reads messages line by line from the server connection
/// - Prints received messages to stdout with proper prompt handling
/// - Exits when the connection is closed or an error occurs
async fn receive_messages(stream: TcpStream) {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Some(line_res) = lines.next().await {
        match line_res {
            Ok(line) => {
                // Print received message and re-print prompt
                println!("\r{}", line);
                print!("> ");
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
            Err(_) => break,
        }
    }
}

#[async_std::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Parse connection parameters with priority: Env Vars -> Command Line Args -> Defaults
    // Usage: client <username> [host] [port]

    let username = env::var("USERNAME").unwrap_or_else(|_| {
        args.get(1)
            .cloned()
            .expect("Username required. Usage: client <username> [host] [port]")
    });

    let host = env::var("HOST").unwrap_or_else(|_| {
        args.get(2)
            .map(|s| s.as_str())
            .unwrap_or("127.0.0.1")
            .to_string()
    });

    let port = env::var("PORT").unwrap_or_else(|_| {
        args.get(3)
            .map(|s| s.as_str())
            .unwrap_or("8080")
            .to_string()
    });

    let addr = format!("{}:{}", host, port);
    println!("Connecting to {} as {}", addr, username);

    // Establish connection to the server
    let mut stream = TcpStream::connect(&addr).await?;

    // Send Connect message to identify ourselves to the server
    let connect_msg = ClientMessage::Connect {
        username: username.clone(),
    };
    let mut json = serde_json::to_string(&connect_msg).unwrap();
    json.push('\n');
    stream.write_all(json.as_bytes()).await?;

    // Spawn background task to handle incoming messages
    let stream_clone = stream.clone();
    task::spawn(receive_messages(stream_clone));

    // Set up stdin reader for user input
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    // Display initial prompt
    print!("> ");
    use std::io::Write;
    std::io::stdout().flush().unwrap();

    // Main interactive loop
    loop {
        line.clear();
        let bytes = reader.read_line(&mut line).await?;
        if bytes == 0 {
            break; // EOF (Ctrl+D)
        }

        let input = line.trim();
        if input.is_empty() {
            // Empty input, just re-prompt
            print!("> ");
            std::io::stdout().flush().unwrap();
            continue;
        }

        // Process user commands
        if input == "leave" {
            // Send leave message to server and exit
            let msg = ClientMessage::Leave {
                username: username.clone(),
            };
            let mut json = serde_json::to_string(&msg).unwrap();
            json.push('\n');
            stream.write_all(json.as_bytes()).await?;
            break;
        } else if let Some(content) = input.strip_prefix("send ") {
            // Send message to chat room
            let msg = ClientMessage::SendMessage {
                username: username.clone(),
                message: content.to_string(),
            };
            let mut json = serde_json::to_string(&msg).unwrap();
            json.push('\n');
            stream.write_all(json.as_bytes()).await?;
        } else {
            // Invalid command, show usage
            println!("Usage: send <message> | leave");
        }

        // Re-display prompt after processing command
        print!("> ");
        std::io::stdout().flush().unwrap();
    }

    Ok(())
}
