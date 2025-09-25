use super::game_logic::Game;
use crate::model::{
    animals::{Animal, AnimalSet, AnimalSetFactory, DefaultAnimalSetFactory},
    money::{money::Money, value::Value, wallet::Wallet},
    player::{
        base_player::Player,
        player_actions::{base_player_actions::PlayerActions, random_actions::RandomPlayerActions},
        player_group::PlayerGroup,
    },
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::{collections::HashMap, sync::Arc, vec};

impl<T> Game<T>
where
    T: PlayerActions,
{
    pub fn new_default_game(player_ids: Vec<String>, players_actions: Vec<T>, seed: u64) -> Self {
        let mut bank_notes: HashMap<Money, u32> = HashMap::new();
        bank_notes.insert(Money::new(Value::new(0)), 2);
        bank_notes.insert(Money::new(Value::new(10)), 4);
        bank_notes.insert(Money::new(Value::new(50)), 1);

        let chicken = DefaultAnimalSetFactory::new(10, vec![0; 4]);
        let goose = DefaultAnimalSetFactory::new(40, vec![0; 4]);
        let cat = DefaultAnimalSetFactory::new(90, vec![0; 4]);
        let dog = DefaultAnimalSetFactory::new(160, vec![0; 4]);
        let sheep = DefaultAnimalSetFactory::new(250, vec![0; 4]);
        let goat = DefaultAnimalSetFactory::new(350, vec![0; 4]);
        let donkey = DefaultAnimalSetFactory::new(500, vec![50, 100, 200, 500]);
        let pig = DefaultAnimalSetFactory::new(650, vec![0; 4]);
        let cow = DefaultAnimalSetFactory::new(800, vec![0; 4]);
        let horse = DefaultAnimalSetFactory::new(1000, vec![0; 4]);

        let game_stack: Vec<Arc<AnimalSet>> = vec![
            Arc::new(chicken),
            Arc::new(goose),
            Arc::new(cat),
            Arc::new(dog),
            Arc::new(sheep),
            Arc::new(goat),
            Arc::new(donkey),
            Arc::new(pig),
            Arc::new(cow),
            Arc::new(horse),
        ];

        let wallet: Wallet = Wallet::new(bank_notes);
        let players: PlayerGroup<T> = PlayerGroup::new(player_ids.clone(), players_actions, wallet);

        Game::new(players, game_stack, seed)
    }

    pub fn new_random_game(player_ids: Vec<String>, players_actions: Vec<T>, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let ratio_player_money: u32 = rng.random_range(1..=3).try_into().unwrap();
        let animals_per_player: u32 = rng.random_range(2..=4).try_into().unwrap();

        let mut bank_notes: HashMap<Money, u32> = HashMap::new();

        let zero = Money::new_u32(0);
        let ten = Money::new_u32(10);
        let fifty = Money::new_u32(50);
        let hundred = Money::new_u32(100);
        let twohundred = Money::new_u32(200);
        let fivehundred = Money::new_u32(500);

        let all_notes = [zero, ten, fifty, hundred, twohundred, fivehundred];

        bank_notes.insert(zero, 2 * ratio_player_money);
        bank_notes.insert(ten, 4 * ratio_player_money);
        bank_notes.insert(fifty, ratio_player_money);

        let mut game_stack: Vec<Arc<AnimalSet>> = Vec::new();

        let min_animal_value: u32 = 10;
        let max_animal_value: u32 = 500;
        let possible_values: Vec<u32> = (min_animal_value..=max_animal_value).step_by(10).collect();

        for _ in &player_ids {
            let use_inflation = rng.random::<f32>() <= 0.1;
            for _ in 0..animals_per_player {
                let random_value_idx = rng.random_range(0..possible_values.len());
                let random_value = possible_values[random_value_idx];

                let count_of_animal = rng.random_range(3..5);
                let mut inflation: Vec<Value> = vec![Value::new(0); count_of_animal];

                if use_inflation {
                    for i in 0..count_of_animal {
                        let random_inflation_idx = rng.random_range(0..all_notes.len());
                        inflation[i] = all_notes[random_inflation_idx].get_value();
                    }
                    inflation.sort();
                }

                let animal_set = DefaultAnimalSetFactory::new_from_value(random_value, inflation);
                game_stack.push(Arc::new(animal_set));
            }
        }

        let wallet: Wallet = Wallet::new(bank_notes);
        let players: PlayerGroup<RandomPlayerActions> = PlayerGroup::new(
            player_ids.clone(),
            players_actions;
            wallet,
        );

        Game::new(players, game_stack, seed)
    }
}
