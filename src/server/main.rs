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

    // Level1 用户注册阶段


    // 开始获取基础信息 建立链接
    info!("Receive Connection from: {}", addr);
    let mut recv_buf = [0u8; 512];
    let size = socket.read(&mut recv_buf).await.unwrap();
    let client: ClientRegisterMessage = bincode::deserialize(&recv_buf[0..size]).unwrap();
    info!("Received the register information: {:?}", client);


    // auth
    let find = conf
        .client
        .iter()
        .find(|x| x.user == client.name && x.secret_key == client.secret);
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

    // 存储当前链接状态
    db.lock().await.insert(uuid.to_string(), ConnectionState {
        last_heart_beat: 0,
        register_time: since_the_epoch.as_millis(),
        name: client.name,
    });

    info!(
        "Client named {} register successfully, uuid: {}",
        find.user,
        uuid.to_string()
    );

    // 返回握手信息
    let response = RegisterResponse::Success {
        uuid: uuid.to_string(),
    };

    let b = bincode::serialize(&response).unwrap();

    // 建立新的链接
    let forward_addr = format!("{}:{}", conf.bind_ip, find.port.to_string());
    let forward_listener = TcpListener::bind(&forward_addr).await;
    if let Err(e) = forward_listener {
        error!("Failed to bind on {} to forward data: {}", forward_addr, e);
        return;
    }
    let forward_listener = forward_listener.unwrap();
    info!("Listen on {} to forward data", forward_addr.to_string());
    match socket.write(b.as_slice()).await {
        Ok(e) => {
            debug!("Write register succ info, size of bytes: {}", e);
        }
        Err(e) => {
            error!("Write Register Response Error: {}", e);
        }
    }

    /**
    首先就是将接收到的数据反序列化成为结构体，然后从conf表中查找是否存在这样的配置，
    如果不存在则写回RegisterResponse::Failed信息。如果成功则新建一个随机的UUID，
    将客户端信息存入前建立的HashMap，在本地建立监听端口(这里监听的是转发数据的端口)，
    建立成功后再向客户端写注册成功信息，这里要附带一个UUID，以待客户端连接转发地址的时候做身份认证。
     */

    // Level2
    loop {
        let (stream, addr) = forward_listener.accept().await.unwrap();
        let mut forward_socket = stream;

        // 查看当前链接 入口IP是否合法
        if !addr.ip().eq(&addr.ip()) {
            forward_socket.write(b"Service Unavaliable. \n").await.unwrap();
            forward_socket.shutdown().await.unwrap();
            return;
        }

        let mut buf = [0u8; 128];
        let re = forward_socket.read(&mut buf).await;
        if let Err(e) = re {
            forward_socket
                .write(b"Service Unavaliable.\n")
                .await
                .unwrap();
            forward_socket.shutdown().await.unwrap();
            break;
        }

        let re = re.unwrap();
        let receive_uuid_str: String = bincode::deserialize(&buf[0..re]).unwrap();
        let receive_uuid = Uuid::parse_str(receive_uuid_str.as_str());
        if let Ok(rUuid) = receive_uuid {
            if uuid.eq(&rUuid) {
                info!(
                            "Client Connect forward port successfully with uuid: {} , name: {}",
                            receive_uuid_str, find.name
                        );
                forward_socket
                    .write_all(bincode::serialize("OK").unwrap().as_ref())
                    .await
                    .unwrap();
                // TODO: handle_forawrd_connection
                // handle_forawrd_connection(forward_socket, forward_listener).await;
            } else {
                info!(
                            "Client send uuid: {} isn't match {}",
                            receive_uuid_str,
                            uuid.to_string()
                        );
                break;
            }
        } else {
            info!(
                        "Client send uuid: {} isn't match {}",
                        receive_uuid_str,
                        uuid.to_string()
                    );
            break;
        }
    }
}
