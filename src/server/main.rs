use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use waterleakage::*;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    println!("WaterLeakage ...");

    let conf = Conf::from_file()?;

    let connection_state = Arc::new(Mutex::new(HashMap::<String, ConnectionState>::new()));
    let addr = format!("{}:{}", conf.bind_ip, conf.server_port);
    let listener = TcpListener::bind(addr.as_str()).await?;
    info!("Server bind on address: {}", addr);

    loop {
        let conf = conf.clone();
        let db = connection_state.clone();
        let (stream, addr) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process().await;
        });
    }

    Ok(())
}


async fn process() {
    println!("hello world");
}
