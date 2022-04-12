use super::user::User;
use crate::event::Event;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct Server {
    listener: UdpSocket,
    users: HashMap<SocketAddr, User>,
}

impl Server {
    pub async fn new(addr: String) -> Server {
        let listener = UdpSocket::bind(addr).await.unwrap();
        let users = HashMap::new();
        Self { listener, users }
    }
    pub async fn run(&mut self) {
        let mut buf = [0u8; 1 << 12];
        loop {
            let v = Some(self.listener.recv_from(&mut buf).await.unwrap());
            if let Some((size, addr)) = v {
                let data: serde_json::value::Value = serde_json::from_slice(&buf[..size]).unwrap();
                self.handle(addr, data).await;
            };
        }
    }
    pub async fn handle(&mut self, addr: SocketAddr, data: serde_json::value::Value) {
        let user = match self.users.contains_key(&addr) {
            true => self.users.get_mut(&addr).unwrap(),
            false => {
                let _user = User::new(addr, "./audio.wav".to_owned());
                self.users.insert(addr, _user);
                self.users.get_mut(&addr).unwrap()
            }
        };
        user.send_receive_status(&mut self.listener, Event::Ok)
            .await;
        user.thread_push(&mut self.listener).await;
    }
}
