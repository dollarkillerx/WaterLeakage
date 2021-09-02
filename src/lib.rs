pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn init_logger() {
    env_logger::init();
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
        let package = Transport{uuid: Uuid::new_v4(), data: BytesMut::new()};

        let bin: Vec<u8> = bincode::serialize(&package).unwrap();

        // let pakcage = bincode::deserialize<Transport>(bin);
    }
}
