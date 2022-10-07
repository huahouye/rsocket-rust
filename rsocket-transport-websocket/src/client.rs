use std::net::SocketAddr;

use rsocket_rust::{async_trait, error::RSocketError, transport::Transport, Result};
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, connect_async, tungstenite::handshake::client::Request};
use url::Url;

use super::connection::WebsocketConnection;

pub type WebsocketRequest = Request;

#[derive(Debug)]
pub(crate) enum Connector {
    Direct(TcpStream),
    Url(Url),
    Request(WebsocketRequest),
}

#[derive(Debug)]
pub struct WebsocketClientTransport {
    connector: Connector,
    websocket_connection: Option<WebsocketConnection>,
}

impl WebsocketClientTransport {
    pub(crate) fn new(connector: Connector) -> WebsocketClientTransport {
        WebsocketClientTransport {
            connector,
            websocket_connection: None,
        }
    }
}

#[async_trait]
impl Transport for WebsocketClientTransport {
    type Conn = WebsocketConnection;

    async fn connect(mut self) -> Result<WebsocketConnection> {
        match self.connector {
            Connector::Direct(stream) => match accept_async(stream).await {
                Ok(ws) => {
                    self.websocket_connection = Some(WebsocketConnection::new(ws));
                    Ok(self.websocket_connection.unwrap())
                }
                Err(e) => Err(RSocketError::Other(e.into()).into()),
            },
            Connector::Url(u) => match connect_async(u).await {
                Ok((stream, _)) => {
                    self.websocket_connection = Some(WebsocketConnection::new(stream));
                    Ok(self.websocket_connection.unwrap())
                }
                Err(e) => Err(RSocketError::Other(e.into()).into()),
            },
            Connector::Request(req) => match connect_async(req).await {
                Ok((stream, _)) => {
                    self.websocket_connection = Some(WebsocketConnection::new(stream));
                    Ok(self.websocket_connection.unwrap())
                }
                Err(e) => Err(RSocketError::Other(e.into()).into()),
            },
        }
    }

    fn get_connection(&self) -> Result<&WebsocketConnection> {
        match &self.websocket_connection {
            Some(wc) => Ok(wc),
            None => Err(RSocketError::WithDescription(
                "get connection err, connection is None".into(),
            )
            .into()),
        }
    }
}

impl From<TcpStream> for WebsocketClientTransport {
    fn from(socket: TcpStream) -> WebsocketClientTransport {
        WebsocketClientTransport::new(Connector::Direct(socket))
    }
}

impl From<&str> for WebsocketClientTransport {
    fn from(addr: &str) -> WebsocketClientTransport {
        let u = if addr.starts_with("ws://") {
            Url::parse(addr).unwrap()
        } else {
            Url::parse(&format!("ws://{}", addr)).unwrap()
        };
        WebsocketClientTransport::new(Connector::Url(u))
    }
}

impl From<SocketAddr> for WebsocketClientTransport {
    fn from(addr: SocketAddr) -> WebsocketClientTransport {
        let u = Url::parse(&format!("ws://{}", addr)).unwrap();
        WebsocketClientTransport::new(Connector::Url(u))
    }
}

impl From<Url> for WebsocketClientTransport {
    fn from(url: Url) -> WebsocketClientTransport {
        WebsocketClientTransport::new(Connector::Url(url))
    }
}

impl From<Request> for WebsocketClientTransport {
    fn from(req: WebsocketRequest) -> Self {
        WebsocketClientTransport::new(Connector::Request(req))
    }
}
