use std::io::stdin;

fn what_is_your_name() -> String {
    let mut your_name = String::new();

    stdin()
        .read_line(&mut your_name)
        .expect("Failed to read line");

    your_name
        .trim()
        .to_lowercase()
}


fn main() {
    let visitors = ["bert", "cory", "dylan"];
    let mut has_permission = false;

    println!("Hello, what's your name?");
    let name = what_is_your_name();
    println!("Hello, {}", name);

    for v in &visitors {
        if v == &name {
            has_permission = true;
        }
    }

    if has_permission {
        println!("Welcome to the Treehouse, {name}");
    } else {
        println!("Sorry, you aren't on the list.");
    }
}
