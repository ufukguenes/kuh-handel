use crate::py_player::py_random_player::RandomPlayerActions;
use kuh_handel_lib::client::Client as CoreClient;
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use tokio::runtime::Runtime;

#[pymodule]
pub fn client_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<Client>();

    Ok(())
}

#[pyclass]
pub struct Client {
    inner: Option<CoreClient>,
    runtime: Runtime,
}

#[pymethods]
impl Client {
    #[new]
    pub fn new(name: String, token: String, bot: &mut RandomPlayerActions) -> Self {
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime");
        Client {
            inner: Some(CoreClient {
                name: name,
                token: token,
                bot: bot.inner.take().unwrap(),
            }),
            runtime: tokio_runtime,
        }
    }

    pub async fn register(&self) {
        let local = tokio::task::LocalSet::new();

        let result = local.block_on(&self.runtime, async {
            self.inner.as_ref().unwrap().register().await
        });

        match result {
            Ok(_) => (),
            Err(err) => println!("{:?}", PyRuntimeError::new_err(format!("{:?}", err))),
        }
    }

    pub async fn start(&mut self) {
        let local = tokio::task::LocalSet::new();

        let result = local.block_on(&self.runtime, async {
            self.inner.take().unwrap().start().await;
        });
    }
}
