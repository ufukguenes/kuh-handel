use model::animals::AnimalSet;
use model::animals::{AnimalSetFactory, DefaultAnimalSetFactory};
use model::game_logic::Game;
mod model;

fn main() {
    let seed: u64 = 0;

    println!("-------Default game--------\n");
    let game: Game<model::player::RandomPlayerActions> = Game::new_default_game(
        vec![
            String::from("ufuk"),
            String::from("leon"),
            String::from("gregor"),
        ],
        seed,
    );
    println!("{}", game);

    println!("");
    println!("---------------------------");
    println!("");

    println!("-------Random game---------\n");
    let game: Game<model::player::RandomPlayerActions> = Game::new_random_game(
        vec![
            String::from("ufuk"),
            String::from("leon"),
            String::from("gregor"),
        ],
        seed,
    );

    println!("{}", game);
}
