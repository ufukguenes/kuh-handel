use model::game_logic::Game;

use crate::model::player::RandomPlayerActions;
mod model;

fn main() {
    let seed: u64 = 0;

    println!("-------Default game--------\n");
    let mut game = Game::new_default_game(
        vec![
            String::from("ufuk"),
            String::from("leon"),
            String::from("gregor"),
        ],
        seed,
    );
    println!("{}", game);
    game.play().unwrap();

    println!("");
    println!("---------------------------");
    println!("");

    println!("-------Random game---------\n");
    let game = Game::new_random_game(
        vec![
            String::from("ufuk"),
            String::from("leon"),
            String::from("gregor"),
        ],
        seed,
    );

    println!("{}", game);
}
