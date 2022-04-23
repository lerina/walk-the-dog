use bracket_lib::prelude::*;

// Storing State
// 
// The game loop runs by calling your application’s tick() function with every
// frame. The tick() function doesn’t know anything about your game, so you
// need a way to store the game’s current status, known as the game state .
// Everything you need to preserve between frames is in your game’s state. 
// The state represents a snapshot of the current game

// Displaying “Hello, World” doesn’t require any data storage, 
// so there’s no need to put variables inside your game state. yet.
struct State {}

// Bracket-lib defines a trait for games state structures named GameState. 
// GameState requires that the object implement a tick() function.
impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        // &mut self allows the tick function to access and change your
        // State instance. ctx provides a window into the currently running 
        // bracket-terminal—accessing information like mouse position 
        // and keyboard input, and sending commands to draw to the window. 
        // 
        // You can think of the tick() function as the bridge between 
        // the game engine and your game
 
        ctx.cls();
        ctx.print(1, 1, "Greetings from Bracket Terminal!");
    }

} 

// The main() function needs to initialize bracket-lib, 
// describing the type of window and game loop you want to create. 
// 
// Setup can fail, so the initialization returns a Result type.
fn main() -> BError {
    // the builder pattern is a common Rust idiom is constructing 
    // complicated objects. 
    // The builder pattern takes advantage of function chaining 
    // to separate many options into individual function calls, 
    // providing more readable code than a giant list of parameters 
    // to a single function.
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?; 
    // When you’ve finished describing the object you want to create, call a build()
    // function. This returns the completed object—or an error—containing your
    // desired system.

    // Now that you’ve created a terminal instance, you need to tell bracket-lib 
    // to start executing the game loop, and link the engine with your State 
    // so that bracket-lib knows where the tick() function is located
    main_loop(context, State{})

    // Any error that occurs will be passed out ofmain—causing your program
    // to crash and display the provided error message. main_loop starts the game
    // loop and begins calling your tick() function on every frame.
}

// Results are Rust’s standard method for handling errors. 
// Results are an enumeration—just like Option and user made  enumerations 
//
// You can handle a Result in three major ways:
//
// 1. Result is an enumeration - you can use match to select a response.
// ```
// match my_result {
//     Ok(a) => do_something_useful(a),
//     Err(a) => handle_error(a),
// }
// ```
//
// 2. Pass errors to the parent function via ?
// ```
// fn my_function() -> BError {
//      ...
//      my_result()?;
// }
// ```
//
// 3. Unwrap like an option - and crash if an error occurred
// ```
// fn my_function() -> BError {
//      ...
//      my_result.unwrap();
// }
// ```

// Calling unwrap is easy, but your program will crash if any error occurs. 
// When you are using many functions that potentially return an error, 
// littering your code with unwrap() calls can also make your code hard to read. 
//
// Adding match statements for every function that might fail also makes for big, 
// hard-to-read code. 
// 
// The ? mark operator can greatly simplify your code and keep it easy to read. 
// The only requirement to use? is that your functionmust return a Result type.















