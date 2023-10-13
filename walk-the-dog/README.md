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

## Transitioning to jumping

Going through each and every change yet again for the jump is redundant. 
Instead, I can recommend you make the following change:

```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(rhb) = self {
            ...
            if keystate.is_pressed("ArrowDown") {
             ...
            }
            if keystate.is_pressed("Space") {
                rhb.jump();
            }

            rhb.update();
        }
    }

}
```

and also

```rust
// src.game.rs

impl RedHatBoy {
    ...
    fn slide(&mut self) {
    ...
    
    fn jump(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Jump);
    }
}
```


You should be able to follow the compiler errors all the way through, creating a transition
from Running to Jumping . 
You can also look up the constant values you need directly out of rhb.json . 
The number of frames is the number of images in Jump in the animation multiplied by 3, 
and subtracting 1, and the name of the animation is `Jump`.

Make sure you handle the `update` event in the transition method for `Jumping`.

Do all that and you see RHB skidding across the ground, doing a kind of dance:

If you've correctly implemented the code for transitioning to the jumping state, our RHB
will play his jumping animation, forever, while skidding across the ground. We've seen
this before with the slide state, so it's time to figure out what's different about jumping. Of
course, we know exactly what's different about jumping – you go up! Well, at least a little.
There are three things we need to do. First, we give RHB vertical velocity when he jumps;
second, we need to add gravity so that RHB will actually come down when he jumps. And
finally, we need to transition running when we land, using our ever-durable state machine:

1. Going up on Jump .
Take a moment and think, where does this belong? Should it go in the update
function, the jump event, or maybe in the enum implementation? 
No, this is a transition change because it happens on jump, 
and it belongs in the `jump` method on the `Running` type . 
You should already have a transition from running to jumping, 
so let's update that function to add vertical velocity:

```rust
// src/game.rs

mod red_hat_boy_states {
    ...
    const JUMP_SPEED: i16 = -25;
    ...
    impl RedHatBoyState<Running> {
        pub fn jump(self) -> RedHatBoyState<Jumping> {
            RedHatBoyState {
               context: self.context.set_vertical_velocity(JUMP_SPEED).reset_frame(),
               _state: Jumping {},
            }
        }
    }


    impl RedHatBoyContext {
        ...
        fn set_vertical_velocity(mut self, y: i16) -> Self {
            self.velocity.y = y;
            self
        }
    }
```

Remember in our 2D coordinate system, y is 0 at the top, 
so we need a negative velocity to go up. 
It also resets the frame so that the jump animation starts at frame 0. 
The implementation in RedHatBoyContext is using the same pattern 
of accepting `mut self` and returning a new `RedHatBoyContext`. 

Now, if you let the app refresh, RHB will take off like Superman!

2. Adding gravity.
In order to have a natural jump, we'll apply gravity on every update. 
We'll do this regardless of state because later, we'll need to have RHB fall 
off of platforms and cliffs, and we don't want to have to constantly pick 
and choose when we're applying gravity. 

This will go in the update function of `RedHatBoyContext` , right at the top:

```rust
// src/game.rs

mod red_hat_boy_states {
    ...
    const GRAVITY: i16 = 1;

    impl RedHatBoyContext {
        fn update(mut self, frame_count: u8) -> Self {
            self.velocity.y += GRAVITY;
```

If you refresh the page right now, you'll get a blink-and-you'll-miss-it problem, and
you'll probably be greeted with a blank screen. The screen isn't really blank; RHB
just fell right through the ground!


We'll need to address this with our first case of collision resolution.

3. Landing on the ground.
This is a bit of a spoiler for the next chapter, but collision detection happens in two
steps. The first is detection, finding places where things collide, and the second is
resolution, where you do something about the collision. Since there isn't anything
to collide with in RHB's empty void, we can just do a simple check in the same
update function to see whether his new position is past the floor and update
the position back to the floor. Keep in mind, you do this after you update to
a new position:

```rust
// src/game.rs

    impl RedHatBoyContext {
        pub fn update(mut self, frame_count: u8) -> Self {
            self.velocity.y += GRAVITY;
            ...
           self.position.x += self.velocity.x;
            self.position.y += self.velocity.y;

            if self.position.y > FLOOR {
                self.position.y = FLOOR;
            }

            self
        }
    ...
```

This may feel redundant, but we can't know gravity pulled RHB past 
the ground without actually calculating where he ends up, 
and we don't draw the in-between state, so the performance cost is minimal. 
This change prevents RHB from falling through the ground and causes a nice jumping arc, 
but he keeps performing the jumping animation for eternity. 

We need to change the state from Jumping back to Running, 
and we need to make that decision in RedHatBoyStateMachine 
because it's a conditional state change based on a condition 
just like the one that transitioned from Sliding to Running .

That's a change to the state machine, much like the one we did for Sliding, 
as seen here:

```rust
// src/game.rs

impl RedHatBoyState<Jumping> {
    ...
    pub fn update(mut self) -> JumpingEndState {
        self.context = self.context.update(JUMPING_FRAMES);
        if self.context.position.y >= FLOOR {
            JumpingEndState::Complete(self.land())
        } else {
            JumpingEndState::Jumping(self)
        }
    }
}
```



So, if the position is on the floor, we need to transition to Running via the stand
method, only we can't! We never wrote a transition from Sliding to Running ,
just the other way around. We also never wrote a JumpingEndState enum, or
a way to convert out of it via From . 
So, right now, you should see several compiler errors about all of that, 
the first being the following:

There's the compiler error, but there's no land method. So, go write it. I'm serious:
go write it yourself. I'm not going to reproduce it here. You can go ahead and follow
along with the previous methods we wrote and implement them. You can do it; I
believe in you. When you do, you'll have a clean animation from Idle to Running ,
then Jumping , and back to Running again. Then, you'll wander off the screen
because we don't have a full scene yet, but we're getting there!


```rust
// src/game.rs

    impl RedHatBoyState<Jumping> {
    ...	
        pub fn land(self) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.reset_frame(),
                _state: Running {},
            }
        }

```

This chapter covered one topic, but one of the most important topics in game
development. 
State machines are everywhere in games, which we saw when we
implemented a small one to manage the Loaded and Loading states of the
WalkTheDog enum itself. 

They are a particularly nice way to implement animation states
that must correspond with what the player is doing, and Rust has great ways to implement
this pattern. 

We used two: the simple one for WalkTheDog, and the much more complex
RedHatBoyStateMachine that uses the typestate pattern. 
The typestate pattern is a commonly used pattern in Rust, 
both inside and outside of game development, so you can expect 
to see it in many Rust projects.

We also used the compiler to drive development, over and over again. 
It's an incredibly useful technique, where you can start with what you want the code to look like 
and use the compiler's error messages to help you fill in the rest of the implementation. 

The code becomes like a paint by numbers picture, 
where you use higher-level code to draw the lines and the compiler error messages 
tell you how to fill them in. 

Rust has very good compiler error messages, getting better with every release, 
and it will pay huge dividends for you to pay close attention to them.

Now that our RHB can run and jump, 
how about he runs and jumps on something? 
We'll put him in a scene and have him jump on it in the next chapter.

