use model::animals::Animal;
mod model;

fn main() {

    let animal = Animal::new(100, 5, [1, 2, 3, 4]);

    println!("Animal value: {}", animal);
}

