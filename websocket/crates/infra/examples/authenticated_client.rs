use futures_util::{SinkExt, StreamExt};
use http::Request;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting authenticated client...");

    let jwt_token = "your_jwt_token_here";
    println!("Using JWT token: {}", jwt_token);

    let url = Url::parse("ws://localhost:8000/room1?123")?;
    println!("Connecting to URL: {}", url);

    // Extract the host from the URL
    let host = url.host_str().ok_or("Invalid URL: missing host")?;

    // Create a request with the Authorization header
    let request = Request::builder()
        .uri(url.as_str())
        .header("Authorization", format!("Bearer {}", jwt_token))
        .header("Host", host)
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", generate_key())
        .body(())?;

    println!("Attempting to connect...");
    match connect_async(request).await {
        Ok((ws_stream, response)) => {
            println!("WebSocket handshake has been successfully completed");
            println!("Response status: {}", response.status());
            println!("Response headers: {:#?}", response.headers());

            let (mut write, mut read) = ws_stream.split();

            // Spawn a task to handle incoming messages
            let read_task = tokio::spawn(async move {
                println!("Starting to listen for messages...");
                while let Some(message) = read.next().await {
                    match message {
                        Ok(msg) => println!("Received: {}", msg),
                        Err(e) => eprintln!("Error receiving message: {}", e),
                    }
                }
                println!("Stopped listening for messages.");
            });

            // Send messages in the main task
            println!("Ready to send messages. Type 'exit' to quit.");
            loop {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim();

                if input == "exit" {
                    println!("Exiting...");
                    break;
                }

                match write.send(Message::Text(input.to_string())).await {
                    Ok(_) => println!("Sent message: {}", input),
                    Err(e) => eprintln!("Error sending message: {}", e),
                }
            }

            // Ensure the read task is properly closed
            read_task.abort();
        }
        Err(e) => {
            eprintln!("Failed to connect: {:?}", e);
            return Err(e.into());
        }
    }

    println!("Client shutting down.");
    Ok(())
}

fn generate_key() -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use rand::Rng;

    let mut key = [0u8; 16];
    rand::thread_rng().fill(&mut key);
    STANDARD.encode(key)
}
