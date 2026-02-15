use super::game_logic::Game;
use kuh_handel_lib::{
    Money,
    animals::AnimalSet,
    player::{base_player::Player, player_actions::PlayerActions, wallet::Wallet},
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::{collections::BTreeMap, sync::Arc, vec};
use tokio::sync::Mutex;

impl Game {
    pub fn new_default_game(
        player_ids: Vec<String>,
        player_actions: Vec<Box<dyn PlayerActions + Send + Sync>>,
        seed: u64,
    ) -> Self {
        let mut bank_notes: BTreeMap<Money, usize> = BTreeMap::new();
        bank_notes.insert(0, 2);
        bank_notes.insert(10, 4);
        bank_notes.insert(50, 1);

        let chicken = AnimalSet::new(10, vec![0; 4]);
        let goose = AnimalSet::new(40, vec![0; 4]);
        let cat = AnimalSet::new(90, vec![0; 4]);
        let dog = AnimalSet::new(160, vec![0; 4]);
        let sheep = AnimalSet::new(250, vec![0; 4]);
        let goat = AnimalSet::new(350, vec![0; 4]);
        let donkey = AnimalSet::new(500, vec![50, 100, 200, 500]);
        let pig = AnimalSet::new(650, vec![0; 4]);
        let cow = AnimalSet::new(800, vec![0; 4]);
        let horse = AnimalSet::new(1000, vec![0; 4]);

        let game_stack: Vec<Arc<Mutex<AnimalSet>>> = vec![
            Arc::new(Mutex::new(chicken)),
            Arc::new(Mutex::new(goose)),
            Arc::new(Mutex::new(cat)),
            Arc::new(Mutex::new(dog)),
            Arc::new(Mutex::new(sheep)),
            Arc::new(Mutex::new(goat)),
            Arc::new(Mutex::new(donkey)),
            Arc::new(Mutex::new(pig)),
            Arc::new(Mutex::new(cow)),
            Arc::new(Mutex::new(horse)),
        ];

        let wallet: Wallet = Wallet::new(bank_notes);

        let players = player_ids
            .iter()
            .zip(player_actions)
            .map(|(id, player_action)| {
                Player::new(
                    id.clone(),
                    wallet.clone(),
                    game_stack.clone(),
                    player_action,
                )
            })
            .collect();

        Game::new(players, wallet, game_stack, seed, 15, 500, 500)
    }

    pub fn new_random_game(
        player_ids: Vec<String>,
        player_actions: Vec<Box<dyn PlayerActions + Send + Sync>>,
        seed: u64,
    ) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let ratio_player_money: usize = rng.random_range(1..=3).try_into().unwrap();
        let animals_per_player: usize = rng.random_range(2..=4).try_into().unwrap();

        let mut bank_notes: BTreeMap<Money, usize> = BTreeMap::new();

        let zero = 0;
        let ten = 10;
        let fifty = 50;
        let hundred = 100;
        let twohundred = 200;
        let fivehundred = 500;

        let all_notes = [zero, ten, fifty, hundred, twohundred, fivehundred];

        bank_notes.insert(zero, 2 * ratio_player_money);
        bank_notes.insert(ten, 4 * ratio_player_money);
        bank_notes.insert(fifty, ratio_player_money);

        let mut game_stack: Vec<Arc<Mutex<AnimalSet>>> = Vec::new();

        let min_animal_value: usize = 10;
        let max_animal_value: usize = 500;
        let possible_values: Vec<usize> =
            (min_animal_value..=max_animal_value).step_by(10).collect();

        for _ in &player_ids {
            let use_inflation = rng.random::<f32>() <= 0.1;
            for _ in 0..animals_per_player {
                let random_value_idx = rng.random_range(0..possible_values.len());
                let random_value = possible_values[random_value_idx];

                let count_of_animal = rng.random_range(3..5);
                let mut inflation: Vec<usize> = vec![0; count_of_animal];

                if use_inflation {
                    for i in 0..count_of_animal {
                        let random_inflation_idx = rng.random_range(0..all_notes.len());
                        inflation[i] = all_notes[random_inflation_idx];
                    }
                    inflation.sort();
                }

                let animal_set = AnimalSet::new(random_value, inflation);
                game_stack.push(Arc::new(Mutex::new(animal_set)));
            }
        }

        let wallet: Wallet = Wallet::new(bank_notes);
        let players = player_ids
            .iter()
            .zip(player_actions)
            .map(|(id, player_action)| {
                Player::new(
                    id.clone(),
                    wallet.clone(),
                    game_stack.clone(),
                    player_action,
                )
            })
            .collect();

        Game::new(players, wallet, game_stack, seed, 15, 500, 500)
    }
}
