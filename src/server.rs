use tokio::{
    net::{TcpListener},
};
use crate::event::Event;
use super::user::User;

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn new(addr: String) -> Self {
        let listener = TcpListener::bind(addr).await.unwrap();
        Self {listener}
    }
    pub async fn accept(&mut self) {
        let (socket, _) = self.listener.accept().await.unwrap();
        let mut user = User::new(socket, "./audio.mp3".to_string()).await;
        tokio::spawn(async move {
            loop {
                let data = user.read().await;
                if Self::handle(&mut user, data).await == Event::Disconnect {
                    user.close();
                    break;
                }
            }
        }).await.unwrap();
    }
    async fn handle(user: &mut User, data: serde_json::value::Value) -> Event {
        match data.get("event") {
            Some(v) => {
                let event = v.as_str().unwrap();
                match event {
                    "stream" => {
                        user.thread_push().await;
                        Event::Stream
                    }
                    "set" => {
                        let path = data.get("track").unwrap().as_str().unwrap();
                        user.load_track(path.to_string()).await;
                        Event::Set
                    }
                    _ => {Event::Disconnect}
                }
            }
            None => {Event::Disconnect}
        }
    }
}
