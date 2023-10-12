## Every little thing I think I see

`WalkTheDog` can be in two states, `Loading` or `Loaded`, after it's initialized. 
Fortunately, we accounted for this when we wrote our `GameLoop`. 
Remember that `GameLoop` returns `Result<Game>` from initialize; 
we're just currently always returning `Ok(WalkTheDog)`. 
What if we made `WalkTheDog` an enum and returned a different state of our game instead? 
That would mean `WalkTheDog` would be a state machine, with two states, 
and initialize would become the transition! 
That's exactly what we're going to do. 

Modify `WalkTheDog` so it is no longer a struct but an enum , as shown here:

```rust
// src/game.rs

/*
pub struct WalkTheDog {
    rhb: Option<RedHatBoy>,
}
*/
pub enum WalkTheDog {
    Loading,
    Loaded(RedHatBoy),
}

```

This is great; now everything is broken! Whoops! 

We'll need to adjust the `WalkTheDog` implementation to account for the two variants. 
First, we'll change the initialize function on WalkTheDog :

```rust
// src/game.rs


#[async_trait(?Send)]
impl Game for WalkTheDog {
    /*
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        let sheet: Option<Sheet> = 
        ...
    */
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                let json = browser::fetch_json("../resources/pix/rhb.json").await?;
                let rhb = RedHatBoy::new(json.into_serde::<Sheet>()?,
                engine::load_image("../resources/pix/rhb.png").await?,);
                
                Ok(Box::new(WalkTheDog::Loaded(rhb)))
            },
            WalkTheDog::Loaded(_) => Err(anyhow!("Error: Game is already initialized!")),
        }
    }//^-- async fn initialize

```

Remember in Chapter 3, Creating a Game Loop, where we made this function return Game? 
This was why!  
In order to ensure initialize is only called once, initialize has to match self on its variants, 
and if we call initialize twice, we'll return an error via `anyhow!`. 
Otherwise, everything inside the `Loading` branch is the same as before, except we return `WalkTheDog::Loaded` instead of `WalkTheDog`. 

This does cause a compiler warning, which will become an error in future versions of Rust 
because `RedHatBoy` isn't public but is exposed in a public type. 
To get rid of that warning, you'll need to make `RedHatBoy` public, and that's fine; 
go ahead and do that. 

```rust
// src/game.rs

pub struct RedHatBoy {
    state_machine: RedHatBoyStateMachine,
    sprite_sheet: Sheet,
    image: HtmlImageElement,
}
```

We also need to change the new constructor to reflect the new type, as shown here:

```rust
// src/game.rs

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog::Loading
    }
}
```

The `WalkTheDog` enum starts in Loading, nothing fancy there. 

The update and draw functions now both need to reflect the changing states; 
you can see those changes here:

```rust
// src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(rhb) = self {
            if keystate.is_pressed("ArrowRight") {
                rhb.run_right();
            }
            if keystate.is_pressed("ArrowDown") {
                rhb.slide();
            }

            rhb.update();
        }
    }
    

```
And also

```rust
// src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...
    fn draw(&self, renderer: &Renderer) {
        ... 
        //self.rhb.as_ref().unwrap().draw(renderer);
        if let WalkTheDog::Loaded(rhb) = self {
            rhb.draw(renderer);
        }
    }//^-- draw()
...
```

You could argue this isn't really a change on the Option type, 
as we still need to check the state of Game each time we operate on rhb, 
and that's true, but I think this more clearly reveals the intent of the system. 

It also has the benefit of getting rid of the `as_ref` , `as_mut` code, 
which is often confusing. 

Now that we've cleaned up that code, let's add one more animation to RHB. 
Let's see this boy jump!

