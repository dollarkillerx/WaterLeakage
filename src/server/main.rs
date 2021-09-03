use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use uuid::Uuid;

use waterleakage::*;

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
            process(stream, addr, conf, db).await;
        });
    }

    // Ok(())
}


async fn process(
    mut socket: TcpStream,
    addr: SocketAddr,
    conf: Conf,
    db: Arc<Mutex<HashMap<String, ConnectionState>>>,
) {
    // print!("hello world");
    // return

    info!("Receive Connection from: {}", addr);
    let mut recv_buf = [0u8; 512];
    let size = socket.read(&mut recv_buf).await.unwrap();
    let client: ClientRegisterMessage = bincode::deserialize(&recv_buf[0..size]).unwrap();
    info!("Received the register information: {:?}", client);

    let find = conf.client
        .iter().find(|x| x.name == client.name && x.secret_key == client.secret);

    if let None = find {
        info!("Client Register Failed: {:?}", client);
        let response = RegisterResponse::Failed {
            reason: "Invalid register configuration.".to_string(),
        };

        let b = bincode::serialize(&response).unwrap();
        match socket.write(b.as_slice()).await {
            Ok(e) => {
                debug!("Write register failed info, size of bytes: {}", e);
            }
            Err(e) => {
                error!("Write Register Response Error: {}", e);
            }
        }

        return;
    }

    let find = find.unwrap();
    let uuid = Uuid::new_v4();
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");

    db.lock().await.insert(uuid.to_string(), ConnectionState {
        last_heart_beat: 0,
        register_time: since_the_epoch.as_millis(),
        name: client.name,
    });

    info!(
        "Client named {} register successfully, uuid: {}",
        find.name,
        uuid.to_string()
    );

    let response = RegisterResponse::Success {
        uuid: uuid.to_string(),
    };

    let b = bincode::serialize(&response).unwrap();

    let forward_addr = format!("{}")
}
