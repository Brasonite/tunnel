use std::str::FromStr;

use ::tunnel::{PublicKey as NativePublicKey, Tunnel as NativeTunnel};
use pollster::FutureExt;
use pyo3::{create_exception, exceptions::PyException, prelude::*, types::PyFunction};

create_exception!(tunnel, PublicKeyParseError, PyException);
create_exception!(tunnel, TunnelCreationError, PyException);
create_exception!(tunnel, TunnelDestroyedError, PyException);
create_exception!(tunnel, TunnelSendingError, PyException);

const TUNNEL_DESTROYED_MSG: &str = "This tunnel has been destroyed.";

#[pyclass]
pub struct PublicKey(NativePublicKey);

#[pymethods]
impl PublicKey {
    #[new]
    fn new(value: &str) -> PyResult<Self> {
        Ok(Self(NativePublicKey::from_str(value).map_err(|e| {
            PublicKeyParseError::new_err(e.to_string())
        })?))
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[pyclass]
pub struct Tunnel {
    pub inner: Option<NativeTunnel>,
}

#[pymethods]
impl Tunnel {
    #[new]
    fn new(handler: Py<PyFunction>) -> PyResult<Self> {
        let inner = NativeTunnel::new(move |sender: NativePublicKey, data: Vec<u8>| {
            Python::attach(|py| handler.call(py, (PublicKey(sender), data), None)).unwrap();
        })
        .block_on()
        .map_err(|e| TunnelCreationError::new_err(e.to_string()))?;

        Ok(Self { inner: Some(inner) })
    }

    fn send(&self, address: &PublicKey, data: &[u8]) -> PyResult<()> {
        let inner = match self.inner.as_ref() {
            Some(inner) => inner,
            None => {
                return Err(TunnelDestroyedError::new_err(TUNNEL_DESTROYED_MSG));
            }
        };

        inner
            .send(address.0, data)
            .block_on()
            .map_err(|e| TunnelSendingError::new_err(e.to_string()))
    }

    fn destroy(&mut self) {
        if let Some(inner) = self.inner.take() {
            inner.destroy().block_on();
        }
    }

    fn close(&self, address: &PublicKey) -> PyResult<()> {
        let inner = match self.inner.as_ref() {
            Some(inner) => inner,
            None => {
                return Err(TunnelDestroyedError::new_err(TUNNEL_DESTROYED_MSG));
            }
        };

        inner.close(address.0);

        Ok(())
    }

    fn close_all(&self) -> PyResult<()> {
        let inner = match self.inner.as_ref() {
            Some(inner) => inner,
            None => {
                return Err(TunnelDestroyedError::new_err(TUNNEL_DESTROYED_MSG));
            }
        };

        inner.close_all();

        Ok(())
    }

    fn sender_address(&self) -> PyResult<PublicKey> {
        let inner = match self.inner.as_ref() {
            Some(inner) => inner,
            None => {
                return Err(TunnelDestroyedError::new_err(TUNNEL_DESTROYED_MSG));
            }
        };

        Ok(PublicKey(inner.sender_address()))
    }

    fn receiver_address(&self) -> PyResult<PublicKey> {
        let inner = match self.inner.as_ref() {
            Some(inner) => inner,
            None => {
                return Err(TunnelDestroyedError::new_err(TUNNEL_DESTROYED_MSG));
            }
        };

        Ok(PublicKey(inner.receiver_address()))
    }
}

#[pymodule]
mod tunnel {
    #[pymodule_export]
    use super::PublicKey;
    use super::Tunnel;
}
