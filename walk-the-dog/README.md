## Adding a UI

### Code recap for the State Machine

```rust
// src/game.rs

pub struct WalkTheDog {
    machine: Option<WalkTheDogStateMachine>,
}

enum WalkTheDogStateMachine {
    Ready(WalkTheDogState<Ready>),
    Walking(WalkTheDogState<Walking>),
    GameOver(WalkTheDogState<GameOver>),
}

impl WalkTheDogStateMachine {
    fn update(self, keystate: &KeyState) -> Self {
        match self {
            WalkTheDogStateMachine::Ready(state) => state.update(keystate).into(),
            WalkTheDogStateMachine::Walking(state) => state.update(keystate).into(),
            WalkTheDogStateMachine::GameOver(state) => state.update().into(),
        }
    }
    fn draw(&self, renderer: &Renderer) {
        match self {
            WalkTheDogStateMachine::Ready(state) => state.draw(renderer),
            WalkTheDogStateMachine::Walking(state) => state.draw(renderer),
            WalkTheDogStateMachine::GameOver(state) => state.draw(renderer),
        }
    }
}

struct WalkTheDogState<T> {
    _state: T,
    walk: Walk,
}


impl<T> WalkTheDogState<T> {
    fn draw(&self, renderer: &Renderer) {
        self.walk.draw(renderer);
    }
}

struct Ready;
struct Walking;
struct GameOver;

impl WalkTheDogState<Ready> {
    fn new() -> WalkTheDogState<Ready> {
        Self
    }

    fn run_right(&mut self) {
        self.walk.boy.run_right();
    }

    fn start_running(mut self) -> WalkTheDogState<Walking> {
        self.run_right();

        WalkTheDogState {
            _state: Walking,
            walk: self.walk,
        }
    }

    fn update(mut self, keystate: &KeyState) -> ReadyEndState {
        self.walk.boy.update();
        if keystate.is_pressed("ArrowRight") {
            ReadyEndState::Complete(self.start_running())
        } else {
            ReadyEndState::Continue(self)
        }
    }
}

impl WalkTheDogState<Walking> {
    fn update(self, keystate: &KeyState) -> WalkTheDogState<Walking> {
        self
    }
    

}

impl WalkTheDogState<GameOver> {
    fn update(self) -> WalkTheDogState<GameOver> {
        self
    }
}

impl From<WalkTheDogState<Ready>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<Ready>) -> Self {
        WalkTheDogStateMachine::Ready(state)
    }
}

impl From<WalkTheDogState<Walking>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<Walking>) -> Self {
        WalkTheDogStateMachine::Walking(state)
    }
}

impl From<WalkTheDogState<GameOver>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<GameOver>) -> Self {
        WalkTheDogStateMachine::GameOver(state)
    }
}

enum ReadyEndState {
    Complete(WalkTheDogState<Walking>),
    Continue(WalkTheDogState<Ready>),
}

impl From<ReadyEndState> for WalkTheDogStateMachine {
    fn from(state: ReadyEndState) -> Self {
        match state {
            ReadyEndState::Complete(walking) => walking.into(),
            ReadyEndState::Continue(ready) => ready.into(),
        }
    }
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog { machine: None }
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self.machine {
            None => {         
                
                let sheet = browser::fetch_json("../resources/pix/rhb.json").await?.into_serde()?;               
                let audio = Audio::new()?;
                let sound = audio.load_sound("../resources/sound/SFX_Jump_23.mp3").await?;
                let background_music = audio.load_sound("../resources/sound/background_song.mp3").await?;

                //play it immediately and drive people nuts                
                audio.play_looping_sound(&background_music)?;

                let rhb = RedHatBoy::new(sheet, 
                                         engine::load_image("../resources/pix/rhb.png").await?,
                                         audio,
                                         sound,);

                let background = engine::load_image("../resources/pix/BG.png").await?;
                let stone = engine::load_image("../resources/pix/Stone.png").await?;
                let tiles = browser::fetch_json("../resources/pix/tiles.json").await?;
                let sprite_sheet = Rc::new(
                                    SpriteSheet::new(
                                        tiles.into_serde::<Sheet>()?,
                                        engine::load_image("../resources/pix/tiles.png").await?,
                                   ));

                let background_width = background.width() as i16;
                
                let starting_obstacles = stone_and_platform(stone.clone(), sprite_sheet.clone(), 0);
                let timeline = rightmost(&starting_obstacles);
                
                let machine = WalkTheDogStateMachine::Ready(WalkTheDogState {
                        _state: Ready,
                        walk: Walk {
                            boy: rhb,
                            backgrounds: [
                                Image::new(background.clone(),
                                Point { x: 0, y: 0 }),
                                Image::new(
                                    background,
                                    Point {
                                        x: background_width,
                                        y: 0,
                                    },
                                ),
                            ],
                            obstacles: starting_obstacles,
                            obstacle_sheet: sprite_sheet,
                            stone,
                            timeline,
                        },
            	});

                Ok(Box::new(WalkTheDog { machine: Some(machine),}))
            },
            Some(_) => Err(anyhow!("Error: Game is already initialized!")),
        }
    }//^-- async fn initialize
    
    fn update(&mut self, keystate: &KeyState) {
        if let Some(machine) = self.machine.take() {
            self.machine.replace(machine.update(keystate));
        }
        assert!(self.machine.is_some());
    }//^-- fn update

    fn draw(&self, renderer: &Renderer) {
        renderer.clear(&Rect::new(Point { x: 0, y: 0 }, 600, 600));

        if let Some(machine) = &self.machine {
            machine.draw(renderer);
        }
    }
} //^-- impl Game for WalkTheDog

fn rightmost(obstacle_list: &Vec<Box<dyn Obstacle>>) -> i16 {
    obstacle_list
        .iter()
        .map(|obstacle| obstacle.right())
        .max_by(|x, y| x.cmp(&y))
        .unwrap_or(0)
}

```
Now we can continue.

### Refactoring initialize

Before we proceed with restoring functionality, you might remember that I said the
creation of `WalkTheDogStateMachine` was "obscured by all the state machine noise."
Specifically, it looked like this:



```rust
// src/game.rs

let machine = WalkTheDogStateMachine::Ready(WalkTheDogState {
                    _state: Ready,
                    walk: Walk {
...
```

To create `WalkTheDogStateMachine` required creating its `Ready` variant 
and passing a `WalkTheDog` state with its `_state` variable set to `Ready`. 

In addition to being noisy, it requires you to remember the correct initial state of the state machine. 

That's what constructors are for!

Let's create a constructor for `WalkTheDogState<Ready>`, as shown here:

```rust
// src/game.rs

impl WalkTheDogState<Ready> {
    fn new(walk: Walk) -> WalkTheDogState<Ready> {
        WalkTheDogState {
            _state: Ready,
            walk,
        }
    }
...

```

This makes it easier to create a new typestate of `WalkTheDogState<Ready>`; 
accepting `Walk`, it needs to be valid. 
Let's also make it easier to create the entire machine, with a smaller constructor:

```rust
// src/game.rs

impl WalkTheDogStateMachine {
    fn new(walk: Walk) -> Self {
        WalkTheDogStateMachine::Ready(WalkTheDogState::new(walk))
    }

    fn update(self, keystate: &KeyState) -> Self {
        ...
...
```

This constructor creates the entire machine with the right state and passes it `Walk`. 
Now that we've made these helper methods, we can make the change to the original initialize
method, making it a little bit easier to read by using the `WalkTheDogStateMachine` constructor:


```rust
// src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self.machine {
            None => { 
                ...
                /*
                let machine = WalkTheDogStateMachine::Ready(WalkTheDogState {
                        _state: Ready,
                        walk: Walk {
                */

                let machine = WalkTheDogStateMachine::new(
                        Walk {
                            boy: rhb,
                            backgrounds: [
                                Image::new(background.clone(),
                                Point { x: 0, y: 0 }),
                                Image::new(
                                    background,
                                    Point {
                                        x: background_width,
                                        y: 0,
                                    },
                                ),
                            ],
                            obstacles: starting_obstacles,
                            obstacle_sheet: sprite_sheet,
                            stone,
                            timeline,
                        },
            	});


```

---------


```rust
// src/game.rs



```


