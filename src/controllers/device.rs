use std::sync::Arc;

use govee_rs::{
    models::{Device, DeviceState, Devices, PowerState},
    Color, GoveeClient,
};

use crate::{
    error::{Error, Result},
    settings::Settings,
    store::RedisStore,
};

#[derive(Clone)]
pub struct DeviceController {
    store: Arc<RedisStore>,
    client: Arc<GoveeClient>,
}

impl DeviceController {
    pub async fn devices(&self) -> Result<Devices> {
        self.store.devices(&self.client).await
    }

    pub async fn state(&self, device: &str) -> Result<DeviceState> {
        let d = self.get_device(device).await?;

        Ok(self.client.state(&d).await?)
    }

    pub async fn toggle(&self, device: &str, state: PowerState) -> Result<()> {
        let d = self.get_device(device).await?;

        self.client.turn(&d, state).await?;

        Ok(())
    }

    pub async fn color(&self, device: &str, color: Color) -> Result<()> {
        let d = self.get_device(device).await?;

        self.client.color(&d, color).await?;

        Ok(())
    }

    async fn get_device(&self, device: &str) -> Result<Device> {
        let devices = self.devices().await?;

        for d in devices.devices {
            if d.device == device {
                return Ok(d);
            }
        }

        Err(Error::DeviceNotFound(device.into()))
    }
}

impl TryFrom<&Settings> for DeviceController {
    type Error = anyhow::Error;

    fn try_from(settings: &Settings) -> std::result::Result<Self, Self::Error> {
        let govee_client = Arc::new(settings.govee_client()?);
        let redis_client = settings.redis_client()?;
        let redis_store = Arc::new(RedisStore::from_client(redis_client, settings.redis_ttl()));

        Ok(Self {
            client: govee_client,
            store: redis_store,
        })
    }
}
