use std::sync::Arc;

use crate::py_player::py_player_actions::{PlayerActions, RustPlayer};
use kuh_handel_lib::client::Client as CoreClient;
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use pyo3_async_runtimes::tokio::future_into_py;
use tokio::sync::Mutex;

#[pymodule]
pub fn client_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<Client>();

    Ok(())
}

#[pyclass]
pub struct Client {
    inner: Arc<Mutex<CoreClient>>,
}

#[pymethods]
impl Client {
    #[new]
    pub fn new(name: String, token: String, bot: Py<PlayerActions>, base_url: String) -> Self {
        let bot = RustPlayer { inner: bot };
        Client {
            inner: Arc::new(Mutex::new(CoreClient::new(
                name,
                token,
                Box::new(bot),
                base_url,
            ))),
        }
    }

    pub fn register<'p>(&mut self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        return future_into_py(py, async move {
            let res = inner.lock().await.register().await;
            match res {
                Ok(_) => Ok(()),
                Err(err) => {
                    println!("{:?}", err); //todo should we do proper error handling here?
                    Ok(())
                }
            }
        });
    }

    pub fn play_one_round<'p>(&mut self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            inner.lock().await.play_one_round("game".to_string()).await;
            Ok(())
        })
    }
}
