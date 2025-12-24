use std::str::FromStr;

use ::tunnel::{PublicKey as NativePublicKey, Tunnel as NativeTunnel};
use pyo3::{
    create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
    sync::PyOnceLock,
};
use tokio::runtime::Runtime;

static PID: PyOnceLock<u32> = PyOnceLock::new();
static RUNTIME: PyOnceLock<Runtime> = PyOnceLock::new();

create_exception!(tunnel, RuntimeMissingError, PyException);
create_exception!(tunnel, PublicKeyParseError, PyException);
create_exception!(tunnel, TunnelCreationError, PyException);
create_exception!(tunnel, TunnelDestroyedError, PyException);
create_exception!(tunnel, TunnelSendingError, PyException);

const RUNTIME_MISSING_MSG: &str = "No initialized Tokio runtime found.";
const TUNNEL_DESTROYED_MSG: &str = "This tunnel was previously destroyed.";

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
    fn new(py: Python, handler: Py<PyAny>) -> PyResult<Self> {
        let inner = runtime(py)?
            .block_on(NativeTunnel::new(
                move |sender: NativePublicKey, data: Vec<u8>| {
                    Python::attach(|py| handler.call(py, (PublicKey(sender), data), None)).unwrap();
                },
            ))
            .map_err(|e| TunnelCreationError::new_err(e.to_string()))?;

        Ok(Self { inner: Some(inner) })
    }

    fn send(&self, py: Python, address: &PublicKey, data: &[u8]) -> PyResult<()> {
        let inner = match self.inner.as_ref() {
            Some(inner) => inner,
            None => {
                return Err(TunnelDestroyedError::new_err(TUNNEL_DESTROYED_MSG));
            }
        };

        runtime(py)?
            .block_on(inner.send(address.0, data))
            .map_err(|e| TunnelSendingError::new_err(e.to_string()))
    }

    fn destroy(&mut self, py: Python) -> PyResult<()> {
        if let Some(inner) = self.inner.take() {
            runtime(py)?.block_on(inner.destroy());
            Ok(())
        } else {
            Err(TunnelDestroyedError::new_err(TUNNEL_DESTROYED_MSG))
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

fn create_tokio_runtime(py: Python) -> PyResult<()> {
    let pid = std::process::id();
    let runtime_pid = *PID.get_or_init(py, || pid);

    if pid != runtime_pid {
        panic!("Attempted to create a new Tokio runtime using a different process.");
    }

    let _ = RUNTIME.set(
        py,
        Runtime::new()
            .map_err(|e| PyValueError::new_err(format!("Could not create Tokio runtime: {e}")))?,
    );

    Ok(())
}

fn runtime<'py>(py: Python<'py>) -> PyResult<&'py Runtime> {
    RUNTIME
        .get(py)
        .ok_or(RuntimeMissingError::new_err(RUNTIME_MISSING_MSG))
}

#[pymodule]
fn pytunnel(m: &Bound<'_, PyModule>) -> PyResult<()> {
    create_tokio_runtime(m.py())?;

    m.add_class::<PublicKey>()?;
    m.add_class::<Tunnel>()?;

    Ok(())
}
