use std::sync::Arc;

use crate::py_player::py_random_player::RandomPlayerActions;
use kuh_handel_lib::client::Client as CoreClient;
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use pyo3_async_runtimes::tokio::future_into_py;

#[pymodule]
pub fn client_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<Client>();

    Ok(())
}

#[pyclass]
pub struct Client {
    inner: Option<Arc<CoreClient>>,
}

#[pymethods]
impl Client {
    #[new]
    pub fn new(
        name: String,
        token: String,
        bot: &mut RandomPlayerActions,
        base_url: String,
    ) -> Self {
        Client {
            inner: Some(Arc::new(CoreClient {
                name: name,
                token: token,
                bot: bot.inner.take().unwrap(),
                base_url: base_url,
            })),
        }
    }

    pub fn register<'p>(&mut self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        match &self.inner {
            Some(inner) => {
                let inner = inner.clone();
                return future_into_py(py, async move {
                    let res = inner.register().await;
                    match res {
                        Ok(_) => Ok(()),
                        Err(err) => {
                            println!("{:?}", err); //todo should we do proper error handling here?
                            Ok(())
                        }
                    }
                });
            }
            None => PyResult::Err(PyRuntimeError::new_err("help")),
        }
    }

    pub fn start<'p>(&mut self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let client_arc = self.inner.take().unwrap();
        let inner = match Arc::try_unwrap(client_arc) {
            Ok(inner) => inner,
            Err(_) => panic!("could not take"),
        };
        future_into_py(py, async move {
            inner.start().await;
            Ok(())
        })
    }
}
