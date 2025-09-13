use super::game_logic::Game;
use crate::model::{
    animals::{AnimalSet, AnimalSetFactory, DefaultAnimalSetFactory},
    money::{Money, Value},
    player::{PlayerGroup, RandomPlayerActions, Wallet},
};
use std::{collections::HashMap, vec};

impl Game<RandomPlayerActions> {
    pub fn new_default_game(player_ids: Vec<String>) -> Self {
        let mut bank_notes: HashMap<Money, u32> = HashMap::new();
        bank_notes.insert(Money::new(Value::new(0)), 2);
        bank_notes.insert(Money::new(Value::new(10)), 4);
        bank_notes.insert(Money::new(Value::new(50)), 1);

        let chicken = DefaultAnimalSetFactory::new(10, vec![0, 4]);
        let goose = DefaultAnimalSetFactory::new(40, vec![0, 4]);
        let cat = DefaultAnimalSetFactory::new(90, vec![0, 4]);
        let dog = DefaultAnimalSetFactory::new(160, vec![0, 4]);
        let sheep = DefaultAnimalSetFactory::new(250, vec![0, 4]);
        let goat = DefaultAnimalSetFactory::new(350, vec![0, 4]);
        let donkey = DefaultAnimalSetFactory::new(500, vec![0, 4]);
        let pig = DefaultAnimalSetFactory::new(650, vec![0, 4]);
        let cow = DefaultAnimalSetFactory::new(800, vec![0, 4]);
        let horse = DefaultAnimalSetFactory::new(1000, vec![0, 4]);

        let game_stack = vec![
            chicken, goose, cat, dog, sheep, goat, donkey, pig, cow, horse,
        ];

        let wallet: Wallet = Wallet::new(bank_notes);
        let players: PlayerGroup<RandomPlayerActions> = PlayerGroup::new(
            player_ids.clone(),
            player_ids.iter().map(|e| RandomPlayerActions {}).collect(),
            wallet,
        );

        Game::new(players, game_stack)
    }
}
