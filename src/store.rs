use anyhow::Context;
use govee_rs::{models::Devices, GoveeClient};
use redis::{aio::Connection, AsyncCommands, Client, RedisResult};

use crate::error::Result;

const DEVICES_KEY: &str = "govee_devices";

#[derive(Debug, Clone)]
pub struct RedisStore {
    client: Client,
    ttl_seconds: usize,
}

impl RedisStore {
    pub fn from_client(client: Client, ttl_seconds: usize) -> Self {
        Self {
            client,
            ttl_seconds,
        }
    }

    pub async fn devices(&self, govee_client: &GoveeClient) -> Result<Devices> {
        // try to get the devices from redis
        // if the key exists, just return the devices from the key
        let mut conn = self
            .connection()
            .await
            .context("Failed to get redis connection")?;

        let v: Option<String> = conn
            .get(DEVICES_KEY)
            .await
            .context("Failed trying to get devices value from redis")?;

        if let Some(ref devices_str) = v {
            tracing::debug!("Redis store hit for '{}'", DEVICES_KEY);
            Ok(
                serde_json::from_str(devices_str)
                    .context("Failed to deserialize stored devices")?,
            )
        } else {
            tracing::debug!("Redis store miss for '{}'", DEVICES_KEY);
            let devices = govee_client
                .devices()
                .await
                .context("Failed to get list of devices from upstream api")?;

            let devices_str =
                serde_json::to_string(&devices).context("Failed to re-serialize devices")?;

            conn.set_ex(DEVICES_KEY, &devices_str, self.ttl_seconds)
                .await
                .context("Failed to set devices key")?;

            Ok(devices)
        }
    }

    async fn connection(&self) -> RedisResult<Connection> {
        self.client.get_tokio_connection().await
    }
}
