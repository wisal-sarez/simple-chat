use async_std::io::{self, BufReader};
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task;
use std::env;
use server::ClientMessage;

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

    // Parse arguments: username, host, port
    // Usage: client <username> [host] [port]
    // Priority: Env Vars -> Command Line Args -> Defaults

    let username = env::var("USERNAME").unwrap_or_else(|_| {
        args.get(1)
            .cloned()
            .expect("Username required. Usage: client <username> [host] [port]")
    });

    let host = env::var("HOST").unwrap_or_else(|_| {
        args.get(2).map(|s| s.as_str()).unwrap_or("127.0.0.1").to_string()
    });

    let port = env::var("PORT").unwrap_or_else(|_| {
        args.get(3).map(|s| s.as_str()).unwrap_or("8080").to_string()
    });

    let addr = format!("{}:{}", host, port);
    println!("Connecting to {} as {}", addr, username);

    let mut stream = TcpStream::connect(&addr).await?;

    // Send Connect message
    let connect_msg = ClientMessage::Connect {
        username: username.clone(),
    };
    let mut json = serde_json::to_string(&connect_msg).unwrap();
    json.push('\n');
    stream.write_all(json.as_bytes()).await?;

    // Spawn task to handle incoming messages
    let stream_clone = stream.clone();
    task::spawn(receive_messages(stream_clone));

    // Handle stdin
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    print!("> ");
    use std::io::Write;
    std::io::stdout().flush().unwrap();

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line).await?;
        if bytes == 0 {
            break; // EOF
        }

        let input = line.trim();
        if input.is_empty() {
            print!("> ");
            std::io::stdout().flush().unwrap();
            continue;
        }

        if input == "leave" {
            let msg = ClientMessage::Leave {
                username: username.clone(),
            };
            let mut json = serde_json::to_string(&msg).unwrap();
            json.push('\n');
            stream.write_all(json.as_bytes()).await?;
            break;
        } else if let Some(content) = input.strip_prefix("send ") {
            let msg = ClientMessage::SendMessage {
                username: username.clone(),
                message: content.to_string(),
            };
            let mut json = serde_json::to_string(&msg).unwrap();
            json.push('\n');
            stream.write_all(json.as_bytes()).await?;
        } else {
            println!("Usage: send <message> | leave");
        }

        print!("> ");
        std::io::stdout().flush().unwrap();
    }

    Ok(())
}