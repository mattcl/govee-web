use std::sync::Arc;

use async_trait::async_trait;
use govee_rs::{models::Devices, GoveeClient};
#[cfg(test)]
use mockall::{automock, predicate::*};
use redis::{aio::Connection, AsyncCommands, Client, RedisError};
use thiserror::Error;

pub type DynDeviceRepo = Arc<dyn DeviceRepo + Send + Sync>;

#[derive(Debug, Error)]
pub enum DeviceRepoError {
    #[error("Device repo health check failed")]
    HealthCheckError {
        #[source]
        source: RedisError,
    },

    #[error("Failed to get a redis connection")]
    ConnectionError {
        #[source]
        source: RedisError,
    },

    #[error("Failed attempting to get devices key from redis")]
    GetKeyError {
        #[source]
        source: RedisError,
    },

    #[error("Failed attempting to set devices key in redis")]
    SetKeyError {
        #[source]
        source: RedisError,
    },

    #[error(transparent)]
    GoveeError(#[from] govee_rs::client::GoveeError),

    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),

    #[error(transparent)]
    UnhandledRedisError(#[from] RedisError),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DeviceRepo {
    async fn list(&self, govee_client: &GoveeClient) -> Result<Devices, DeviceRepoError>;
    async fn health_check(&self) -> Result<(), DeviceRepoError>;
}

const DEVICES_KEY: &str = "govee_devices";
const HEALTH_KEY: &str = "govee_health_check";

#[derive(Debug, Clone)]
pub struct RedisDeviceRepo {
    client: Client,
    ttl_seconds: usize,
}

impl RedisDeviceRepo {
    pub fn from_client(client: Client, ttl_seconds: usize) -> Self {
        Self {
            client,
            ttl_seconds,
        }
    }

    async fn connection(&self) -> Result<Connection, DeviceRepoError> {
        self.client
            .get_tokio_connection()
            .await
            .map_err(|source| DeviceRepoError::ConnectionError { source })
    }
}

#[async_trait]
impl DeviceRepo for RedisDeviceRepo {
    async fn list(&self, govee_client: &GoveeClient) -> Result<Devices, DeviceRepoError> {
        // try to get the devices from redis
        // if the key exists, just return the devices from the key
        let mut conn = self.connection().await?;

        let v: Option<String> = conn
            .get(DEVICES_KEY)
            .await
            .map_err(|source| DeviceRepoError::GetKeyError { source })?;

        if let Some(ref devices_str) = v {
            tracing::debug!("Redis store hit for '{}'", DEVICES_KEY);
            Ok(serde_json::from_str(devices_str)?)
        } else {
            tracing::debug!("Redis store miss for '{}'", DEVICES_KEY);
            let devices = govee_client.devices().await?;

            let devices_str = serde_json::to_string(&devices)?;

            conn.set_ex(DEVICES_KEY, &devices_str, self.ttl_seconds)
                .await
                .map_err(|source| DeviceRepoError::SetKeyError { source })?;

            Ok(devices)
        }
    }

    async fn health_check(&self) -> Result<(), DeviceRepoError> {
        let mut conn = self.connection().await?;

        conn.set(HEALTH_KEY, "hello")
            .await
            .map_err(|source| DeviceRepoError::HealthCheckError { source })
    }
}
