use crate::{Money, Value};
use kuh_handel_lib::player::{
    wallet::Affordability as CoreAffordability, wallet::Wallet as CoreWallet,
};
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[pymodule]
pub fn wallet_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<Wallet>();
    m.add_class::<Affordability>();

    Ok(())
}

#[pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub inner: CoreWallet,
}

#[pymethods]
impl Wallet {
    #[new]
    pub fn new(bank_notes: BTreeMap<Money, usize>) -> Self {
        Wallet {
            inner: CoreWallet::new(bank_notes),
        }
    }

    pub fn withdraw(&mut self, amount: Vec<Money>) -> PyResult<()> {
        match self.inner.withdraw(&amount) {
            Ok(_) => Ok(()),
            Err(err) => Err(PyRuntimeError::new_err(format!("{:?}", err))),
        }
    }

    pub fn deposit(&mut self, amount: Vec<Money>) {
        self.inner.deposit(&amount);
    }

    pub fn total_money(&self) -> Value {
        self.inner.total_money()
    }

    pub fn check_if_exact(&self, bill_combination: Vec<Money>) -> bool {
        self.inner.check_if_exact(&bill_combination)
    }

    pub fn propose_bill_combinations(
        &self,
        amount: Value,
        also_suggest_smaller_values: bool,
    ) -> Vec<(Value, Vec<Money>)> {
        self.inner
            .propose_bill_combinations(amount, also_suggest_smaller_values)
    }

    pub fn can_afford(&self, payment_amount: Vec<Money>) -> Affordability {
        match self.inner.can_afford(&payment_amount) {
            CoreAffordability::Exact => Affordability::Exact(),
            CoreAffordability::Alternative(items) => Affordability::Alternative(items),
            CoreAffordability::CannotAfford => Affordability::CannotAfford(),
        }
    }

    pub fn bank_notes(&self) -> &BTreeMap<Money, usize> {
        self.inner.bank_notes()
    }
}

impl Wallet {
    pub fn convert_to_rs(self) -> CoreWallet {
        self.inner
    }

    pub fn convert_to_py(wallet: &CoreWallet) -> Self {
        Wallet {
            inner: wallet.clone(),
        }
    }
}

#[pyclass]
pub enum Affordability {
    Exact(),
    Alternative(Vec<Money>),
    CannotAfford(),
}
