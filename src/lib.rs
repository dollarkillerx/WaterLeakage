pub use log::{debug, error, info};
pub use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use conf::*;

mod conf;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn init_logger() {
    env_logger::init();
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RegisterResponse {
    Success { uuid: String },
    Failed { reason: String },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConnectionState {
    pub last_heart_beat: i32,
    pub register_time: u128,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ClientRegisterMessage {
    pub name: String,
    pub secret: String,
    pub protocol: String,
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use log::info;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use super::*;

    #[test]
    fn hello_word() {
        println!("hello world");
    }

    #[test]
    fn log() {
        init_logger();
        info!("hello world");
        info!("hello world");
        info!("hello world");
        println!("trr");
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct Transport {
        uuid: Uuid,
        data: BytesMut,
    }

    #[test]
    fn test_bincode() {
        let package = Transport { uuid: Uuid::new_v4(), data: BytesMut::new() };

        let bin: Vec<u8> = bincode::serialize(&package).unwrap();

        // let pakcage = bincode::deserialize<Transport>(bin);
    }
}
