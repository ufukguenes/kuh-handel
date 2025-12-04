use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{Money, Value, player::player_error::PlayerError};
use pyo3::prelude::*;
use std::collections::BTreeMap;

#[pyclass(unsendable)]
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Wallet {
    #[serde_as(as = "Vec<(_, _)>")]
    bank_notes: BTreeMap<Money, usize>,
}

impl Wallet {
    pub fn new(bank_notes: BTreeMap<Money, usize>) -> Self {
        Wallet {
            bank_notes: bank_notes,
        }
    }

    pub fn add_money(&mut self, money: Money) {
        self.bank_notes
            .entry(money)
            .and_modify(|curr| *curr += 1)
            .or_insert(1);
    }

    pub fn withdraw(&mut self, amount: &Vec<Money>) -> Result<(), PlayerError> {
        let backup_notes = self.bank_notes.clone();

        for money in amount {
            let count = self.bank_notes.get(&money);
            match count {
                Some(&count) => {
                    let new_count: isize = count as isize - 1 as isize;
                    if new_count > 0 {
                        self.bank_notes.insert(*money, new_count as usize);
                    } else if new_count == 0 {
                        self.bank_notes.remove(money);
                    } else {
                        self.bank_notes = backup_notes;
                        return Result::Err(PlayerError::MoneyNotAvailable);
                    }
                }
                None => {
                    self.bank_notes = backup_notes;
                    return Result::Err(PlayerError::MoneyNotAvailable);
                }
            }
        }

        Ok(())
    }

    pub fn deposit(&mut self, amount: &Vec<Money>) {
        for money in amount {
            self.bank_notes
                .entry(*money)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    pub fn to_vec(&self) -> Vec<Money> {
        let mut all_bills = Vec::new();
        for (money, count) in &self.bank_notes {
            let current_bills = vec![money; count.clone()];
            all_bills.extend(current_bills);
        }

        all_bills
    }

    pub fn total_money(&self) -> Value {
        let mut total: Value = 0;

        for (money, amount) in &self.bank_notes {
            total += money * amount;
        }
        total
    }

    pub fn check_if_exact(&self, bill_combination: &Vec<Money>) -> bool {
        let mut temp_wallet = self.clone();
        temp_wallet.withdraw(bill_combination).is_ok()
    }

    pub fn propose_bill_combinations(
        &self,
        amount: Value,
        also_suggest_smaller_values: bool,
    ) -> Vec<(Value, Vec<Money>)> {
        //todo test this
        let mut available_bills: Vec<&Money> = self.bank_notes.keys().clone().collect();
        available_bills.sort();

        let mut possible_payments: Vec<(usize, Vec<Money>)> = vec![(0, Vec::new())];

        for (bill, count) in &self.bank_notes {
            if *bill >= amount {
                possible_payments.push((*bill, vec![bill.clone()]));
                break;
            }

            let mut check_against_these_combinations = possible_payments.clone();
            for _ in 0..count.clone() {
                let mut new_combinations: Vec<(usize, Vec<Money>)> = Vec::new();
                for (old_value, combination) in &check_against_these_combinations {
                    let mut new_bills = combination.clone();
                    new_bills.push(bill.clone());

                    let new_value = old_value + bill;

                    new_combinations.push((new_value, new_bills));

                    if new_value >= amount {
                        break;
                    }
                }

                possible_payments.extend(new_combinations.clone());
                check_against_these_combinations = new_combinations;
            }
        }

        let mut out: Vec<(Value, Vec<Money>)> = possible_payments
            .into_iter()
            .filter(|(val, _)| also_suggest_smaller_values || *val >= amount)
            .map(|(val, combination)| (val, combination))
            .collect();
        out.sort_by(|(a, _), (b, _)| a.cmp(b));

        out
    }

    pub fn can_afford(&self, payment_amount: &Vec<Money>) -> Affordability {
        let total_payed: Value = payment_amount.iter().map(|money| money).sum();
        let total_owned = self.total_money();

        if total_owned < total_payed {
            return Affordability::CannotAfford();
        };

        let fits_exact = self.check_if_exact(&payment_amount);

        if fits_exact {
            return Affordability::Exact(payment_amount.clone());
        }

        // just pick the bill combination with the smallest overhead
        let alternative = self
            .propose_bill_combinations(total_payed, false)
            .get(0)
            .unwrap() // can not fail as test for smaller amount is rejected above
            .1
            .clone();
        return Affordability::Alternative(alternative);
    }

    pub fn bank_notes(&self) -> &BTreeMap<Money, usize> {
        &self.bank_notes
    }
}

#[pyclass(unsendable)]
pub enum Affordability {
    Exact(Vec<Money>),
    Alternative(Vec<Money>),
    CannotAfford(),
}
