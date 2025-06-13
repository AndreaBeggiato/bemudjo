use std::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2323").await?;
    println!("Bemudjo MUD Server listening on 127.0.0.1:2323");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);
        
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("Error handling client {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(mut socket: TcpStream) -> io::Result<()> {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    writer.write_all(b"Welcome to Bemudjo MUD!\r\n").await?;
    writer.write_all(b"Type 'help' for available commands or 'quit' to exit.\r\n").await?;
    writer.write_all(b"> ").await?;

    loop {
        line.clear();
        
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {
                let command = line.trim();
                
                match command {
                    "quit" | "exit" => {
                        writer.write_all(b"Goodbye!\r\n").await?;
                        break;
                    }
                    "help" => {
                        writer.write_all(b"Available commands:\r\n").await?;
                        writer.write_all(b"  help - Show this help message\r\n").await?;
                        writer.write_all(b"  look - Look around\r\n").await?;
                        writer.write_all(b"  say <message> - Say something\r\n").await?;
                        writer.write_all(b"  quit - Exit the game\r\n").await?;
                    }
                    "look" => {
                        writer.write_all(b"You are in a simple room. There's nothing much to see here yet.\r\n").await?;
                    }
                    cmd if cmd.starts_with("say ") => {
                        let message = &cmd[4..];
                        writer.write_all(format!("You say: {}\r\n", message).as_bytes()).await?;
                    }
                    "" => {
                    }
                    _ => {
                        writer.write_all(b"Unknown command. Type 'help' for available commands.\r\n").await?;
                    }
                }
                
                if !line.trim().is_empty() {
                    writer.write_all(b"> ").await?;
                }
            }
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                break;
            }
        }
    }

    Ok(())
}