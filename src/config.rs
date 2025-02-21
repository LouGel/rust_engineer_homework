use crate::error::{Error, Result};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub ethereum_rpc_url: String,
    pub cache_duration: Duration,
    pub host: IpAddr,
    pub port: u16,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let _ = dotenv::dotenv();

        let ethereum_rpc_url = match std::env::var("ETHEREUM_RPC_URLS") {
            Ok(val) => val,
            Err(_) => {
                return Err(Error::Config("No Ethereum RPC URLs provided".into()));
            }
        };

        let cache_duration_secs = std::env::var("CACHE_DURATION_SECONDS")
            .unwrap_or_else(|_| "0".into())
            .parse::<u64>()
            .map_err(|_| Error::Config("Invalid CACHE_DURATION_SECS".into()))?;

        let host = std::env::var("HOST")
            .unwrap_or_else(|_| "0.0.0.0".into())
            .parse::<IpAddr>()
            .map_err(|_| Error::Config("Invalid HOST".into()))?;

        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse::<u16>()
            .map_err(|_| Error::Config("Invalid PORT".into()))?;

        let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into());

        Ok(Self {
            ethereum_rpc_url,
            cache_duration: Duration::from_secs(cache_duration_secs),
            host,
            port,
            log_level,
        })
    }

    pub fn server_address(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}
