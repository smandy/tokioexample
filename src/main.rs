use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    // Create a channel for sending messages to all clients
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Spawn a task to handle incoming messages and broadcast them to all clients
    tokio::spawn(broadcast_messages(rx));

    // Start the TCP listener
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server listening on 127.0.0.1:8080");

    // Accept incoming connections
    while let Ok((socket, _)) = listener.accept().await {
        // Clone the sender for the new client
        let tx_clone = tx.clone();

        // Spawn a task to handle the client
        tokio::spawn(handle_client(socket, tx_clone));
    }
}

async fn handle_client(mut socket: TcpStream, tx: mpsc::Sender<String>) {
    // Clone the socket for reading and writing
    let (mut reader, mut writer) = socket.split();

    // Read the client's username
    let mut username_buf = [0; 64];
    let n = reader.read(&mut username_buf).await.unwrap();
    let username = String::from_utf8_lossy(&username_buf[..n]).into_owned();

    println!("User '{}' connected.", username);

    // Send a welcome message to the client
    tx.send(format!("Welcome, {}!\n", username)).await.unwrap();

    // Clone the sender for the client's messages
    let tx_clone = tx.clone();

    // Spawn a task to handle incoming messages from the client
    tokio::spawn(async move {
        loop {
            let mut buf = [0; 1024];
            let n = reader.read(&mut buf).await.unwrap();
            if n == 0 {
                // Client disconnected
                println!("User '{}' disconnected.", username);
                break;
            }

            let message = String::from_utf8_lossy(&buf[..n]).into_owned();
            tx_clone.send(format!("{}: {}", username, message)).await.unwrap();
        }
    });

    // Broadcast messages from other clients to this client
    while let Some(message) = tx.recv().await {
        writer.write_all(message.as_bytes()).await.unwrap();
    }
}

async fn broadcast_messages(mut rx: mpsc::Receiver<String>) {
    loop {
        // Wait for a message from any client
        if let Some(message) = rx.recv().await {
            // Broadcast the message to all clients
            println!("Broadcasting: {}", message);
        } else {
            // The last sender has been dropped, no more clients
            break;
        }
    }
}
