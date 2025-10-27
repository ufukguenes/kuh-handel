use std::collections::BTreeMap;
use std::rc::Rc;

use kuh_handel_lib::player::base_player::Player as CorePlayer;
use kuh_handel_lib::player::random_player::RandomPlayerActions as CoreRandomPlayerActions;
use pyo3::prelude::*;

use crate::py_animals::{Animal, AnimalSet};
use crate::py_player::py_wallet::Wallet;
use crate::PlayerId;

#[pymodule]
pub fn base_player_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<Player>();

    Ok(())
}

#[pyclass(unsendable)]
pub struct Player {
    inner: CorePlayer,
}

#[pymethods]
impl Player {
    #[new]
    pub fn new(id: String, wallet: Wallet, game_stack: Vec<AnimalSet>) -> Self {
        let dummy_action = CoreRandomPlayerActions::new(id.clone(), 0);
        let core_wallet = wallet.convert_to_rs();

        let game_stack = game_stack
            .iter()
            .map(|set| Rc::new(set.convert_to_rs()))
            .collect();

        let core_player = CorePlayer::new(id, core_wallet, game_stack, Box::new(dummy_action));
        Player { inner: core_player }
    }

    pub fn id(&self) -> &PlayerId {
        &self.inner.id()
    }

    pub fn wallet(&mut self) -> Wallet {
        Wallet::convert_to_py(self.inner.wallet())
    }

    pub fn owned_animals(&self) -> BTreeMap<Animal, usize> {
        self.inner
            .owned_animals()
            .iter()
            .map(|(animal, &count)| (Animal::convert_to_py(animal.clone()), count))
            .collect()
    }

    pub fn add_animals(&mut self, animal: &Animal, count: usize) {
        self.inner.add_animals(animal.convert_to_rs(), count);
    }

    pub fn remove_animals(&mut self, animal: &Animal, count: usize) -> bool {
        self.inner
            .remove_animals(animal.convert_to_rs(), count)
            .is_ok()
    }
}
