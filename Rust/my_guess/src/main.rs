use std::io::stdin;
use rand::Rng;

fn main() {
    println!("what is your guess?");
    let mut guess = String::new();

    stdin().read_line(&mut guess)
        .expect("could not read line");

}
