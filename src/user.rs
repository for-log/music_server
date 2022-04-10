use minimp3::{Decoder, Frame, Error};
use serde::{Serialize};
use serde_json::json;
use tokio::{
    io::{AsyncWriteExt, AsyncReadExt},
    net::{TcpStream},
    fs::File
};
use super::event::Event;
use super::constants::*;

pub struct User {
    socket: TcpStream,
    audio: Decoder<File>,
    bytes_len: usize,
}

impl User {
    pub async fn new(socket: TcpStream, path: String) -> Self {
        let file = File::open(path).await.unwrap();
        let audio = Decoder::new(file);
        Self {socket, audio, bytes_len: 0}
    }
    pub async fn thread_push(&mut self) {
        match self.audio.next_frame_future().await {
            Ok(Frame {
                   data,
                   sample_rate,
                   channels,
                   ..
               }) => {
                   self.send(&Self::create_event(Event::Data, &Self::serialize_data(&data, sample_rate, channels))).await;
                   self.bytes_len += data.len() / channels;
                }
            Err(Error::Eof) => self.send(&Self::create_event(Event::End, &"END")).await,
            Err(e) => panic!("Error in thread_push: {:?}", e),
        }
    }
    pub fn serialize_data(data: &Vec<i16>, sample_rate: i32, channels: usize) -> serde_json::value::Value {
        let mut result = vec![0u8; data.len()];
        let mut minus_index = vec![];
        for i in 0..data.len() {
            if data[i] < 0 {
                minus_index.push(i);
            }
            result[i] = data[i].abs() as u8;
        }
        json!({
            "data": result,
            "munises": minus_index,
            "sample_rate": sample_rate,
            "channels": channels
        })
    }
    pub fn create_event<T: Serialize>(event: Event, data: &T) -> serde_json::value::Value {
        json!({
            "event": event as u32,
            "data": data
        })
    }
    pub async fn send(&mut self, data: &serde_json::value::Value) {
        self.socket.write_all(format!("{}{}", data.to_string(), ENDBYTES).as_bytes()).await;
    }
    pub async fn send_receive_status(&mut self, status: Event) {
        let data = json!({
            "event": "receive_status",
            "status": status as u32
        });
        self.socket.write_all(data.to_string().as_bytes()).await;
    }
    pub async fn read(&mut self) -> serde_json::value::Value {
        let mut data = [0u8; 1 << 12];
        match self.socket.read(&mut data).await {
            Ok(n) => {
                if n == 0 {
                    return json!({"event": "disconnect"});
                }
                self.send_receive_status(Event::Ok).await;
                serde_json::from_slice(&data[0..n]).unwrap()
            },
            Err(_) => panic!("Error in read user!")
        }
    }
    pub async fn load_track(&mut self, path: String) {
        let file = File::open(path).await.unwrap();
        let audio = Decoder::new(file);
        self.audio = audio;
        self.bytes_len = 0;
    }
    pub fn close(&mut self) {
        self.socket.shutdown(std::net::Shutdown::Both).unwrap();
    }
}

