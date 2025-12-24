use std::str::FromStr;

use ::tunnel::{PublicKey as NativePublicKey, Tunnel as NativeTunnel};
use futures::{SinkExt, StreamExt, channel::mpsc::channel};
use js_sys::{Function, Uint8Array};
use wasm_bindgen::prelude::*;

struct DataEvent {
    sender: NativePublicKey,
    data: Vec<u8>,
}

#[wasm_bindgen]
pub struct PublicKey(NativePublicKey);

#[wasm_bindgen]
impl PublicKey {
    #[wasm_bindgen(constructor)]
    pub fn new(value: &str) -> Result<Self, JsError> {
        Ok(PublicKey(
            NativePublicKey::from_str(value).map_err(|e| JsError::new(&e.to_string()))?,
        ))
    }
}

/// A tunnel used to send and receive data.
#[wasm_bindgen]
pub struct Tunnel(NativeTunnel);

#[wasm_bindgen]
impl Tunnel {
    /// Creates a new tunnel using the provided callback.
    pub async fn new(handler: Function) -> Result<Self, JsError> {
        let (tx, mut rx) = channel::<DataEvent>(32);

        let inner = NativeTunnel::new(move |sender: NativePublicKey, data: Vec<u8>| {
            let mut tx_clone = tx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                tx_clone.send(DataEvent { sender, data }).await.unwrap();
            });
        })
        .await
        .map_err(|e| JsError::new(&e.to_string()))?;

        wasm_bindgen_futures::spawn_local(async move {
            while let Some(event) = rx.next().await {
                handler
                    .call2(
                        &JsValue::null(),
                        &JsValue::from(PublicKey(event.sender)),
                        &JsValue::from(Uint8Array::from(event.data.as_slice())),
                    )
                    .unwrap();
            }
        });

        Ok(Self(inner))
    }

    /// Sends some data to another tunnel, given the provided address is valid.
    ///
    /// **Note:** if a tunnel is not currently connected to the receiver, it
    /// will first attempt to estabilish a connection.
    ///
    /// # Arguments
    ///
    /// - `address`: The **receiver address** of the tunnel to send data to.
    /// - `data`: The data to be sent.
    pub async fn send(&self, address: &PublicKey, data: &Uint8Array) -> Result<(), JsError> {
        self.0
            .send(address.0, &data.to_vec())
            .await
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Closes both the sender and the receiver endpoint and consumes this object.
    ///
    /// Ideally, this should be called before the execution of the program ends
    /// or before a tunnel is discarded.
    pub async fn destroy(self) {
        self.0.destroy().await
    }

    /// Closes a connection to another tunnel, if it exists.
    pub fn close(&self, address: &PublicKey) {
        self.0.close(address.0);
    }

    /// Closes all connections between this tunnel and other tunnels.
    pub fn close_all(&self) {
        self.0.close_all();
    }

    /// Returns the address of the sender endpoint of this tunnel.
    ///
    /// The sender enpoint is responsible for sending data to other tunnels.
    /// As such, when sending data, this address will be cited as the source.
    pub fn sender_address(&self) -> PublicKey {
        PublicKey(self.0.sender_address())
    }

    /// Returns the address of the receiver endpoint of this tunnel.
    ///
    /// The receiver enpoint is responsible for receiving data from other tunnels.
    /// As such, senders should send data to this address.
    pub fn receiver_address(&self) -> PublicKey {
        PublicKey(self.0.receiver_address())
    }
}
