// filename: main.rs

fn main() {
    let x = 20;
    let y = 1;
    let answer = add_me( double_me(x),  double_me(y) );
    
    println!("{} is the answer", answer);

    //-------------------------------------------
    let name = "str literal";
    println!("{}", &name[4..]);
}

fn  add_me(x: i32, y: i32) -> i32 {

    x+y
}

fn double_me(a: i32) -> i32 {

    a*2
}
