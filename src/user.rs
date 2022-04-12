use super::constants::*;
use super::event::Event;
use serde::Serialize;
use serde_json::json;
use std::fs::File;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use wav::header::Header;

const diff_size: usize = 1 << 12;

pub struct User {
    addr: SocketAddr,
    audio: Vec<i16>,
    header_audio: Header,
    current_bits: usize,
    pub is_close: bool,
}

impl User {
    pub fn new(addr: SocketAddr, path: String) -> User {
        let mut file = File::open(path).unwrap();
        let (header_audio, a) = wav::read(&mut file).unwrap();
        let audio = a.try_into_sixteen().unwrap().to_vec();
        Self {
            addr,
            audio,
            header_audio,
            current_bits: 0,
            is_close: false,
        }
    }
    pub async fn thread_push(&mut self, writter: &mut UdpSocket) {
        if self.current_bits >= self.audio.len() {
            self.send(
                writter,
                &Self::create_event(Event::End, &"END")
            ).await;
            return;
        }
        let data = self.audio[self.current_bits..self.current_bits + diff_size].to_vec();
        self.send(
            writter,
            &Self::create_event(Event::Data, &self.serialize_data(&data)),
        )
        .await;
        self.current_bits += diff_size;
    }
    pub fn serialize_data<Z: serde::Serialize>(&self, data: &Z) -> serde_json::value::Value {
        json!({ 
            "data": data,
            "format": self.header_audio.audio_format,
            "channels": self.header_audio.channel_count,
            "rate": self.header_audio.sampling_rate
         })
    }
    pub fn create_event<Z: Serialize>(event: Event, data: &Z) -> serde_json::value::Value {
        json!({
            "event": event as u32,
            "data": data
        })
    }
    pub async fn send(&mut self, writter: &mut UdpSocket, data: &serde_json::value::Value) {
        let result_data = format!("{}{}", data.to_string(), ENDBYTES);
        self.send_receive_len(writter, result_data.len()).await;
        match writter.send_to(result_data.as_bytes(), self.addr).await {
            Ok(_) => {}
            Err(_) => self.close(),
        };
    }
    pub async fn send_receive_status(&mut self, writter: &mut UdpSocket, status: Event) {
        let data = Self::create_event(Event::Status, &(status as u32));
        match writter
            .send_to(data.to_string().as_bytes(), self.addr)
            .await
        {
            Ok(_) => {}
            Err(_) => self.close(),
        };
    }
    pub async fn send_receive_len(&mut self, writter: &mut UdpSocket, ln: usize) {
        let data = Self::create_event(Event::Len, &ln);
        match writter
            .send_to(data.to_string().as_bytes(), self.addr)
            .await
        {
            Ok(_) => {}
            Err(_) => self.close(),
        };
    }
    pub async fn load_track(&mut self, path: String) {
        let mut file = File::open(path).unwrap();
        let (header_audio, a) = wav::read(&mut file).unwrap();
        let audio = a.try_into_sixteen().unwrap().to_vec();
        self.audio = audio;
        self.header_audio = header_audio;
    }
    pub fn close(&mut self) {
        self.is_close = true;
    }
}
