use model::animals::AnimalSet;
use model::animals::{AnimalSetFactory, DefaultAnimalSetFactory};
use model::game_logic::Game;
mod model;

fn main() {
    let animal_set: AnimalSet = DefaultAnimalSetFactory::new(500, vec![0, 4]);
    let seed: u64 = 0;
    let game: Game<model::player::RandomPlayerActions> = Game::new_random_game(
        vec![
            String::from("ufuk"),
            String::from("leon"),
            String::from("gregor"),
        ],
        seed,
    );
    println!("Animal value: {}", animal_set);

    println!("{}", game);
}
