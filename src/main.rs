mod constants;
mod event;
mod server;
mod user;

#[tokio::main]
async fn main() {
    let mut server = server::Server::new("127.0.0.1:8080".to_string()).await;
    server.run().await;
}
