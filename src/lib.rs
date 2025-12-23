use std::{fmt::Debug, sync::Arc};

use anyhow::{Result, anyhow};
use dashmap::DashMap;
use iroh::{
    Endpoint, PublicKey,
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler, Router},
};

pub const ALPN: &[u8] = b"brasonite/tunnel/v1";

/// A trait implemented for objects which can handle incoming data from a tunnel.
///
/// For convenience's sake, this trait is implemented for function pointers. As
/// such, any function which takes a [PublicKey] and a `Vec<u8>` in this order
/// can be used as a [DataHandler].
pub trait DataHandler: 'static + Send + Sync {
    fn process_incoming_data(&self, sender: PublicKey, data: Vec<u8>);
}

impl<Func> DataHandler for Func
where
    Func: 'static + Send + Sync + Fn(PublicKey, Vec<u8>) -> (),
{
    fn process_incoming_data(&self, sender: PublicKey, data: Vec<u8>) {
        self(sender, data)
    }
}

pub struct TunnelProtocol {
    pub handler: Option<Arc<dyn DataHandler>>,
}

impl TunnelProtocol {
    pub fn new() -> Self {
        Self { handler: None }
    }

    pub fn with_handler(mut self, handler: Arc<dyn DataHandler>) -> Self {
        self.handler = Some(handler);
        self
    }
}

impl ProtocolHandler for TunnelProtocol {
    async fn accept(&self, connection: Connection) -> std::result::Result<(), AcceptError> {
        let handler = match &self.handler {
            Some(handler) => handler,
            None => return Ok(()),
        };

        while let Ok(mut stream) = connection.accept_uni().await {
            let data = stream.read_to_end(usize::MAX).await.unwrap();
            handler.process_incoming_data(connection.remote_id(), data);
        }

        Ok(())
    }
}

impl Debug for TunnelProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tunnel").finish()
    }
}

/// A tunnel used to send and receive data.
#[derive(Debug)]
pub struct Tunnel {
    pub sender: Endpoint,
    pub receiver: Router,

    connections: DashMap<PublicKey, Connection>,
}

impl Tunnel {
    /// Creates a new tunnel using the provided [DataHandler] object.
    pub async fn new<T: DataHandler>(handler: T) -> Result<Self> {
        let sender = Endpoint::bind().await?;
        let receiver_endpoint = Endpoint::bind().await?;

        let protocol = Arc::new(TunnelProtocol::new().with_handler(Arc::new(handler)));

        let receiver = Router::builder(receiver_endpoint)
            .accept(ALPN, Arc::clone(&protocol))
            .spawn();

        sender.online().await;
        receiver.endpoint().online().await;

        Ok(Self {
            sender,
            receiver,

            connections: DashMap::new(),
        })
    }

    /// Sends some data to another tunnel, given the provided address is valid.
    ///
    /// **Note:** if a tunnel is not currently connected to the receiver, it
    /// will first attempt to estabilish a connection.
    ///
    /// # Arguments
    ///
    /// - `address`: The **receiver address** of the tunnel to send data to.
    ///  Can be any value which can be converted to a [PublicKey].
    /// - `data`: The data to be sent.
    /// This data can be anything representable as a slice of bytes.
    pub async fn send(&self, address: impl Into<PublicKey>, data: impl AsRef<[u8]>) -> Result<()> {
        let address = address.into();

        let receiver = if let Some(connection) = self.connections.get(&address) {
            connection
        } else {
            let connection = self.sender.connect(address, ALPN).await?;
            self.connections.insert(address, connection);

            self.connections.get(&address).unwrap()
        };

        let mut stream = receiver.open_uni().await?;
        stream.write_all(data.as_ref()).await?;
        stream.finish()?;

        if let Some(error) = stream.stopped().await? {
            return Err(anyhow!("Failed to send data. Error code: {}", error));
        }

        Ok(())
    }

    /// Closes both the sender and the receiver endpoint and consumes this object.
    ///
    /// Ideally, this should be called before the execution of the program ends.
    pub async fn destroy(self) {
        self.sender.close().await;
        self.receiver.shutdown().await.unwrap();
    }

    /// Closes a connection to another tunnel.
    pub fn close(&self, address: PublicKey) {
        self.connections
            .remove(&address)
            .inspect(|(_, connection)| connection.close(0u32.into(), b"user_request"));
    }

    /// Closes all connections between this tunnel and other tunnels.
    pub fn close_all(&self) {
        self.connections
            .iter()
            .for_each(|connection| connection.close(0u32.into(), b"user_request"));

        self.connections.clear();
    }

    /// Returns the address of the sender endpoint of this tunnel.
    ///
    /// The sender enpoint is responsible for sending data to other tunnels.
    /// As such, when sending data, this address will be cited as the source.
    pub fn sender_address(&self) -> PublicKey {
        self.sender.id()
    }

    /// Returns the address of the receiver endpoint of this tunnel.
    ///
    /// The receiver enpoint is responsible for receiving data from other tunnels.
    /// As such, senders should send data to this address.
    pub fn receiver_address(&self) -> PublicKey {
        self.receiver.endpoint().id()
    }
}
