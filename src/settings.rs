use std::{
    fmt::{Debug, Display},
    net::{Ipv4Addr, SocketAddr},
};

use anyhow::{Context, Result};
use figment::{providers::Env, Figment};
use govee_rs::{GoveeClient, DEFAULT_API_URL};
use serde::Deserialize;
use url::Url;

fn default_ttl() -> usize {
    300
}

fn default_api_url() -> Url {
    DEFAULT_API_URL.parse().unwrap()
}

fn default_bind() -> Ipv4Addr {
    Ipv4Addr::new(127, 0, 0, 1)
}

fn default_port() -> u16 {
    3000
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct Settings {
    api_key: String,
    redis_uri: Url,

    #[serde(default = "default_ttl")]
    redis_ttl_seconds: usize,

    #[serde(default = "default_bind")]
    bind_addr: Ipv4Addr,

    #[serde(default = "default_port")]
    port: u16,

    #[serde(default = "default_api_url")]
    remote_api_url: Url,
}

impl Settings {
    /// Attempt to create a new settings instance.
    ///
    /// This will read from the environment and fail if any required keys are
    /// not specified.
    pub fn new() -> Result<Self> {
        let s = Figment::new().merge(Env::prefixed("GOVEE_")).extract()?;

        Ok(s)
    }

    pub fn redis_ttl(&self) -> usize {
        self.redis_ttl_seconds
    }

    /// Get the socket address as specified by the settings.
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(std::net::IpAddr::V4(self.bind_addr), self.port)
    }

    pub fn govee_client(&self) -> Result<GoveeClient> {
        Ok(GoveeClient::new(
            self.remote_api_url.as_str(),
            &self.api_key,
        )?)
    }

    pub fn redis_client(&self) -> Result<redis::Client> {
        redis::Client::open(self.redis_uri.as_str()).context("Failed to connect to redis")
    }
}

impl Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Settings: ")?;
        writeln!(f, "  api_key:        ******")?;
        writeln!(f, "  remote_api_url: {}", &self.remote_api_url)?;
        writeln!(f, "  redis_uri:      {}", &self.redis_uri)?;
        writeln!(f, "  bind_addr:      {}", &self.bind_addr)?;
        write!(f, "  port:           {}", self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation() {
        let expected = Settings {
            api_key: "foo".into(),
            redis_uri: "redis://fakeredis:1234".try_into().unwrap(),
            redis_ttl_seconds: 300,
            bind_addr: Ipv4Addr::new(127, 0, 0, 1),
            port: 3000,
            remote_api_url: default_api_url(),
        };

        let settings = temp_env::with_vars(
            [
                ("GOVEE_API_KEY", Some("foo")),
                ("GOVEE_REDIS_URI", Some("redis://fakeredis:1234")),
            ],
            || Settings::new(),
        )
        .unwrap();

        assert_eq!(settings, expected);

        let expected = Settings {
            api_key: "bar".into(),
            redis_uri: "redis://otherredis:1234".try_into().unwrap(),
            redis_ttl_seconds: 400,
            bind_addr: Ipv4Addr::new(255, 255, 255, 255),
            port: 7070,
            remote_api_url: "http://foo/bar".try_into().unwrap(),
        };

        let settings = temp_env::with_vars(
            [
                ("GOVEE_API_KEY", Some("bar")),
                ("GOVEE_REDIS_URI", Some("redis://otherredis:1234")),
                ("GOVEE_REDIS_TTL_SECONDS", Some("400")),
                ("GOVEE_BIND_ADDR", Some("255.255.255.255")),
                ("GOVEE_PORT", Some("7070")),
                ("GOVEE_REMOTE_API_URL", Some("http://foo/bar")),
            ],
            || Settings::new(),
        )
        .unwrap();

        assert_eq!(settings, expected);
    }

    #[test]
    fn socket_addr() {
        let expected = SocketAddr::from(([127, 0, 0, 1], 3000));

        let settings = temp_env::with_vars(
            [
                ("GOVEE_API_KEY", Some("foo")),
                ("GOVEE_REDIS_URI", Some("redis://fakeredis:1234")),
            ],
            || Settings::new(),
        )
        .unwrap();

        assert_eq!(settings.socket_addr(), expected);
    }
}
