use std::io::stdin;
use rand::{thread_rng, Rng};

/*NOTE:  clippy::comparison_chain is telling us to use cmp.
 *
use std::cmp::Ordering;

match guess.cmp(&secret_number) {
    Ordering::Less => println!("Too small!"),
    Ordering::Greater => println!("Too big!"),
    Ordering::Equal => {
        println!("You win!");
        break;
    }
}
*/
enum Status {
    Hi,
    Low,
    Correct
}

fn main() {
    let mut gen = thread_rng();
    let number = gen.gen_range(1..101);
    
    println!("what is your guess?");
    loop {
        let mut guess = String::new();
        stdin().read_line(&mut guess)
            .expect("could not read line");
        let guess: i32 = guess.trim().parse().expect("is this an integer?");
        
        println!("your number is {guess}");
        let ans = if guess == number {
            Status::Correct
        } else if guess < number {
            Status::Low
        } else {
            Status::Hi 
        };

        match ans {
            Status::Correct => {println!("you win"); break},
            Status::Low => println!("Too low!"),
            Status::Hi => println!("Too hight")
        }
    }
}
