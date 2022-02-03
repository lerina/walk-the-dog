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
        ctx.print(1, 1, "Hello, Bracket Terminal!");
    }

} 

// The main() function needs to initialize bracket-lib, 
// describing the type of window and game loop you want to create. 
// 
// Setup can fail, so the initialization returns a Result type.
fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?;

    main_loop(context, State{})
}
