mod user;
mod event;
mod server;
mod constants;


#[tokio::main]
async fn main() {
    let mut server = server::Server::new("127.0.0.1:8080".to_string()).await;
    tokio::spawn(async move {
        loop {
            server.accept().await;
        }
    }).await;
}