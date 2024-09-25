use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "ws://localhost:8000/room1";

    let (ws_stream, _) = connect_async(url).await?;
    println!("WebSocket handshake has been successfully completed");

    let (mut write, mut read) = ws_stream.split();

    // Spawn a task to handle incoming messages
    tokio::spawn(async move {
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => println!("Received: {}", msg),
                Err(e) => eprintln!("Error receiving message: {}", e),
            }
        }
    });

    // Send messages in the main task
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "exit" {
            break;
        }

        write.send(Message::Text(input.to_string())).await?;
    }

    Ok(())
}
