use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::model::game_errors::GameError;
use crate::model::money::money::Money;
use crate::model::money::value::Value;
use std::collections::HashMap;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    #[serde_as(as = "Vec<(_, _)>")]
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
            .find(|(key, _)| key.value() >= amount)
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

    pub fn total_money(&self) -> Value {
        let mut total: u32 = 0;
        for (money, amount) in &self.bank_notes {
            total += money.as_u32() * amount;
        }
        Value::new(total)
    }

    pub fn propose_bill_combinations(&self, amount: Value) -> Vec<(Value, Vec<Money>)> {
        //todo test this
        let mut available_bills: Vec<&Money> = self.bank_notes.keys().clone().collect();
        available_bills.sort();

        let mut possible_payments: Vec<(u32, Vec<Money>)> = vec![(0, Vec::new())];

        for (bill, count) in &self.bank_notes {
            if bill.value() >= amount {
                possible_payments.push((bill.as_u32(), vec![bill.clone()]));
                break;
            }

            let mut check_against_these_combinations = possible_payments.clone();
            for _ in 0..count.clone() {
                let mut new_combinations: Vec<(u32, Vec<Money>)> = Vec::new();
                for (old_value, combination) in &check_against_these_combinations {
                    let mut new_bills = combination.clone();
                    new_bills.push(bill.clone());

                    let new_value = old_value + bill.as_u32();

                    new_combinations.push((new_value, new_bills));

                    if new_value >= amount.value() {
                        break;
                    }
                }

                possible_payments.extend(new_combinations.clone());
                check_against_these_combinations = new_combinations;
            }
        }

        let mut out: Vec<(Value, Vec<Money>)> = possible_payments
            .into_iter()
            .filter(|(val, _)| *val >= amount.value())
            .map(|(val, combination)| (Value::new(val), combination))
            .collect();
        out.sort_by(|(a, _), (b, _)| a.cmp(b));

        out
    }

    pub fn can_afford(&self, payment_amount: &Vec<Money>) -> Affordability {
        let total_payed: u32 = payment_amount.iter().map(|money| money.as_u32()).sum();
        let total_owned = self.total_money();

        if total_owned.value() < total_payed {
            return Affordability::CannotAfford;
        };

        let mut mut_wallet = self.bank_notes.clone();
        for money in payment_amount {
            let count = mut_wallet.get_mut(&money);
            match count {
                Some(count) => {
                    if *count > 0 {
                        *count -= 1;
                    } else {
                        break;
                    }
                }
                None => break,
            }

            return Affordability::Exact;
        }

        // just pick the bill combination with the smallest overhead
        let alternative = self
            .propose_bill_combinations(Value::new(total_payed))
            .get(0)
            .unwrap()
            .1
            .clone();
        Affordability::Alternative(alternative)
    }
}

pub enum Affordability {
    Exact,
    Alternative(Vec<Money>),
    CannotAfford,
}
