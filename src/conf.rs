use super::*;

#[derive(Debug, Deserialize, Clone)]
pub struct Conf {
    pub bind_ip: String,
    pub server_port: u16,
    pub client: Vec<ClientConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientConfig {
    pub user: String,
    pub port: u16,
    pub protocol: String,
    pub secret_key: String,
}

impl Conf {
    pub fn from_file() -> Result<Conf> {
        if let Ok(cf) = std::fs::read_to_string("server.toml") {
            match toml::from_str(cf.as_str()) {
                Ok(conf) => {
                    info!("Read Config: {:#?}", conf);
                    Ok(conf)
                }
                Err(e) => {
                    error!("Error while read server.toml");
                    error!("{:#?}", e);
                    Err(e.into())
                }
            }
        } else {
            error!("Cannot read config file.");
            Err("Cannot read config file.".into())
        }
    }
}
