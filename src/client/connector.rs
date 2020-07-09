use async_native_tls::{TlsConnector, TlsStream};
use futures::prelude::*;
use std::time::Duration;
use tokio::net::TcpStream;
use url::Url;

pub(crate) use async_native_tls::Error as TlsError;
pub(crate) use tokio::io::Error as SocketError;

pub enum Error {
    Timeout,
    Socket(SocketError),
    Tls(TlsError),
}

pub(crate) struct Config {
    connection_timeout: Option<Duration>,
    keepalive_timeout: Option<Duration>,
    nodelay: Option<bool>,
}

pub(crate) struct GeminiConnector<'u> {
    config: Config,
    url: &'u Url,
}

impl<'u> GeminiConnector<'u> {
    pub fn new(config: Config, url: &'u Url) -> Self {
        GeminiConnector { config, url }
    }

    async fn connect(self) -> Result<TlsStream<TcpStream>, Error> {
        let host = self.url.host_str().expect("host is required");
        let port = self.url.port().expect("port is required");

        let connect = TcpStream::connect((host, port));
        let stream = match self.config.connection_timeout {
            Some(timeout) => {
                tokio::time::timeout(timeout, connect)
                    .map_err(|_| Error::Timeout)
                    .await??
            }
            None => connect.await?,
        };

        if let Some(nodelay) = self.config.nodelay {
            stream.set_nodelay(nodelay)?;
        }

        stream.set_keepalive(self.config.keepalive_timeout)?;
        let tls_stream = TlsConnector::new()
            .use_sni(true)
            .danger_accept_invalid_certs(true) // TODO: TOFU
            .connect(host, stream)
            .await?;

        Ok(tls_stream)
    }
}

impl From<tokio::io::Error> for Error {
    fn from(err: tokio::io::Error) -> Self {
        Error::Socket(err)
    }
}

impl From<async_native_tls::Error> for Error {
    fn from(err: async_native_tls::Error) -> Self {
        Error::Tls(err)
    }
}
