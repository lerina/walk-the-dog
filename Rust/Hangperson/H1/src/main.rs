//Step 1 
use rand::thread_rng;
use rand::seq::SliceRandom;
use std::io::{self, Write};

// SliceRandom  
//              Extension trait on slices, providing random mutation and sampling methods.
//              This trait is implemented on all [T] slice types, providing several methods for choosing and shuffling elements.
// fn choose<R>(&self, rng: &mut R) -> Option<&Self::Item> where R: Rng + ?Sized 
//              Returns a reference to one random element of the slice, or None if the slice is empty.

// [source](https://docs.rs/rand/latest/rand/seq/trait.SliceRandom.html)

fn main() {
    // cls
    //print!("{}[2J", 27 as char);
    print!("\x1B[2J");
    io::stdout().flush().unwrap();

    let word_list = ["aardvark", "baboon", "camel"];

    //TODO-1 - Randomly choose a word from the word_list and assign it to a variable called chosen_word.
    let mut rng = thread_rng();
    let secret_word = word_list.choose(&mut rng).unwrap();

    println!("DEBUG: {}", secret_word);
    //TODO-2 - Ask the user to guess a letter and assign their answer to a variable called guess. Make guess lowercase.
    use std::io;
    print!("Guess a letter: ");
    io::stdout().flush().unwrap();
    let mut user_guess = String::new();
    io::stdin()
            .read_line(&mut user_guess)
            .expect("Failed to read input");
  //   std::io::stdout().flush();
    //let user_guess = user_guess.chars().next().unwrap(); //[0 as u8];
    let user_guess = user_guess.chars().next().unwrap().to_lowercase().to_string(); //[0 as u8];
    

    //TODO-3 - Check if the letter the user guessed (guess) is one of the letters in the chosen_word.
        if secret_word.contains(&user_guess) {
            println!("Right");
        } else {
            println!("Wrong");
        }
   

}
