use serde::{Deserialize, Serialize};

use crate::model::game_errors::GameError;
use crate::model::money::money::Money;
use crate::model::money::value::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    bank_notes: HashMap<Money, u32>,
}

impl Wallet {
    pub fn new(bank_notes: HashMap<Money, u32>) -> Self {
        Wallet {
            bank_notes: bank_notes,
        }
    }

    pub fn withdraw(&mut self, amount: Value) -> Result<(), GameError> {
        // ToDo: implement the actual version of withdraw (check money and maybe receive not a value but a handful of money)

        let key = self
            .bank_notes
            .iter()
            .find(|(key, _)| key.get_value() >= amount)
            .map(|(key, _)| *key);
        match key {
            Some(k) => {
                self.bank_notes
                    .entry(k)
                    .and_modify(|e| *e = e.checked_sub(1).or(Some(0)).unwrap());
            }
            None => (),
        };
        Ok(())
    }
}
