use model::animals::{DefaultAnimalFactory, AnimalFactory};
use model::animals::AnimalSet;
mod model;

fn main() {
    let animal_set: AnimalSet = DefaultAnimalFactory::new(500, vec![0, 4]); 

    println!("Animal value: {}", animal_set);
}

