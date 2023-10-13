use anyhow::{Result, anyhow};
use async_trait::async_trait;

use web_sys::HtmlImageElement;

/*
use std::{collections::HashMap, rc::Rc, sync::Mutex};
use crate::{ 
    browser, 
    engine::{self, Game, Point, Rect, Renderer, Sheet, KeyState},};
*/
use crate::{
    browser,
    engine::{self, Game, KeyState, Rect, Renderer, Sheet},
};
use self::red_hat_boy_states::*;


pub enum WalkTheDog {
    Loading,
    Loaded(RedHatBoy),
}

impl WalkTheDog {

    pub fn new() -> Self {
        WalkTheDog::Loading {}
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {

    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                let sheet = browser::fetch_json("../resources/pix/rhb.json").await?.into_serde()?;
                let rhb = RedHatBoy::new(sheet, engine::load_image("../resources/pix/rhb.png").await?);
               
                Ok(Box::new(WalkTheDog::Loaded(rhb)))
            },
            WalkTheDog::Loaded(_) => Err(anyhow!("Error: Game is already initialized!")),
        }
    }//^-- async fn initialize


    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(rhb) = self {
            if keystate.is_pressed("ArrowRight") {
                rhb.run_right();
            }
            if keystate.is_pressed("ArrowDown") {
                rhb.slide();
            }
            if keystate.is_pressed("Space") {
                rhb.jump();
            }

            rhb.update();
        }
    }

    fn draw(&self, renderer: &Renderer) {
        renderer.clear( &Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 600.0,
                        height: 600.0,
        });

        if let WalkTheDog::Loaded(rhb) = self {
            rhb.draw(renderer);
        }
    }//^-- draw()

}//^-- impl Game for WalkTheDog

//--------------------------------------------

pub enum Event {
    Run,
    Slide,
    Update,
    Jump,
}

pub struct RedHatBoy {
    state_machine: RedHatBoyStateMachine,
    sprite_sheet: Sheet,
    image: HtmlImageElement,
}


impl RedHatBoy {
    fn new(sheet: Sheet, image: HtmlImageElement) -> Self {
        RedHatBoy {
            state_machine: RedHatBoyStateMachine::Idle(RedHatBoyState::new()),
            sprite_sheet: sheet,
            image,
        }
    }


    fn draw(&self, renderer: &Renderer) {
        let frame_name = format!("{} ({}).png", 
                                 self.state_machine.frame_name(),
                                 (self.state_machine.context().frame / 3) + 1);
        let sprite = self.sprite_sheet
                        .frames
                        .get(&frame_name)
                        .expect("Cell not found");

        renderer.draw_image(
                            &self.image,
                            &Rect {
                                x: sprite.frame.x.into(),
                                y: sprite.frame.y.into(),
                                width: sprite.frame.w.into(),
                                height: sprite.frame.h.into(),
                            },
                            &Rect {
                                x: self.state_machine.context()
                                .position.x.into(),
                                y: self.state_machine.context()
                                .position.y.into(),
                                width: sprite.frame.w.into(),
                                height: sprite.frame.h.into(),
                            },);
        
    }//^-- fn draw

    fn update(&mut self) {
        self.state_machine = self.state_machine.update();
    }

    fn run_right(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Run);
    }

    fn slide(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Slide);
    }

    fn jump(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Jump);
    }

}//^-- impl RedHatBoy


//----------

#[derive(Copy, Clone)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    Running(RedHatBoyState<Running>),
    Sliding(RedHatBoyState<Sliding>),
    Jumping(RedHatBoyState<Jumping>),
}


impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            (RedHatBoyStateMachine::Idle(state), Event::Run) => state.run().into(), 
            (RedHatBoyStateMachine::Running(state), Event::Slide) => state.slide().into(),
            (RedHatBoyStateMachine::Running(state), Event::Jump) => state.jump().into(),
            (RedHatBoyStateMachine::Idle(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Running(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Sliding(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Jumping(state), Event::Update) => state.update().into(),
            _ => self,
        }
    }

    fn frame_name(&self) ->&str {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.frame_name(),
            RedHatBoyStateMachine::Running(state) => state.frame_name(),
            RedHatBoyStateMachine::Sliding(state) => state.frame_name(),
            RedHatBoyStateMachine::Jumping(state) => state.frame_name(),
        }
    }


    fn context(&self) ->&RedHatBoyContext {
        match self {
            RedHatBoyStateMachine::Idle(state) =>&state.context(),
            RedHatBoyStateMachine::Running(state) =>&state.context(),
            RedHatBoyStateMachine::Sliding(state) => &state.context(),
            RedHatBoyStateMachine::Jumping(state) => state.context(),
        }
    }

    fn update(self) -> Self {
        self.transition(Event::Update)
    }

}//^-- impl


impl From<RedHatBoyState<Idle>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Idle>) -> Self {
        RedHatBoyStateMachine::Idle(state)
    }
}

impl From<RedHatBoyState<Running>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Running>) -> Self {
        RedHatBoyStateMachine::Running(state)
    }
}

impl From<RedHatBoyState<Sliding>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Sliding>) -> Self {
        RedHatBoyStateMachine::Sliding(state)
    }
}

impl From<RedHatBoyState<Jumping>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Jumping>) -> Self {
        RedHatBoyStateMachine::Jumping(state)
    }
}



impl From<SlidingEndState> for RedHatBoyStateMachine {
    fn from(end_state: SlidingEndState) -> Self {
        match end_state {
            SlidingEndState::Running(running) => running_state.into(),
            SlidingEndState::Sliding(sliding) => sliding_state.into(),
        }
    }
}

impl From<JumpingEndState> for RedHatBoyStateMachine {
    fn from(state: JumpingEndState) -> Self {
        match state {
            JumpingEndState::Jumping(jumping) => jumping.into(),
            JumpingEndState::Landing(landing) => landing.into(),
        }
    }
}

mod red_hat_boy_states {
    use crate::engine::Point;
    
    const FLOOR: i16 = 475;
    const IDLE_FRAME_NAME: &str = "Idle";
    const RUN_FRAME_NAME: &str = "Run";
    const SLIDING_FRAME_NAME: &str = "Slide";
    const JUMPING_FRAME_NAME: &str = "Jump";

    const IDLE_FRAMES: u8 = 29;
    const RUNNING_FRAMES: u8 = 23;
    const SLIDING_FRAMES: u8 = 14;
    const JUMPING_FRAMES: u8 = 35;
    const RUNNING_SPEED: i16 = 3;
    const JUMP_SPEED: i16 = -25;
    const GRAVITY: i16 = 1;

    #[derive(Copy, Clone)]
    pub struct Idle;

    impl RedHatBoyState<Idle> {

        pub fn new() -> Self {
            RedHatBoyState {
                context: RedHatBoyContext {
                        frame: 0,
                        position: Point { x: 0, y: FLOOR },
                        velocity: Point { x: 0, y: 0 },
                },
                _state: Idle {},
            }
        }//^-- fn new

        pub fn frame_name(&self) -> &str {
            IDLE_FRAME_NAME
        }

        pub fn run(self) -> RedHatBoyState<Running> {
            RedHatBoyState {
               context: self.context.reset_frame().run_right(),
                _state: Running {},
            }
        }

        pub fn update(mut self) -> Self {
            self.context = self.context.update(IDLE_FRAMES);
            self
        }

    }//^-- impl RedHatBoyState<Idle>

    #[derive(Copy, Clone)]
    pub struct Running;

    impl RedHatBoyState<Running> {
        pub fn frame_name(&self) -> &str {
            RUN_FRAME_NAME
        }

        pub fn update(mut self) -> Self {
            self.context = self.context.update(RUNNING_FRAMES);
            self
        }

        pub fn slide(self) -> RedHatBoyState<Sliding> {
            RedHatBoyState {
                context: self.context.reset_frame(),
                _state: Sliding {},
            }
        }

        pub fn jump(self) -> RedHatBoyState<Jumping> {
            RedHatBoyState {
               context: self.context.set_vertical_velocity(JUMP_SPEED).reset_frame(), //self.context.reset_frame().set_vertical_velocity(JUMP_SPEED),
               _state: Jumping {},
            }
        }

    }

    #[derive(Copy, Clone)]
    pub struct Sliding;

    pub enum SlidingEndState {
        Running(RedHatBoyState<Running>),
        Sliding(RedHatBoyState<Sliding>),
    }


    impl RedHatBoyState<Sliding> {
        pub fn frame_name(&self) -> &str {
            SLIDING_FRAME_NAME
        }

        pub fn update(mut self) -> SlidingEndState {
            self.context = self.context.update(SLIDING_FRAMES);

            if self.context.frame >= SLIDING_FRAMES {
                SlidingEndState::Running(self.stand())
            } else {
                SlidingEndState::Sliding(self)
            }
        }

        pub fn stand(self) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.reset_frame(),
                _state: Running,
            }
        }

    }//^-- impl RedHatBoyState<Sliding>

    #[derive(Copy, Clone)]
    pub struct Jumping;

    pub enum JumpingEndState {
        Jumping(RedHatBoyState<Jumping>),
        Landing(RedHatBoyState<Running>),
    }

    impl RedHatBoyState<Jumping> {
        pub fn frame_name(&self) -> &str {
            JUMPING_FRAME_NAME
        }

        pub fn update(mut self) -> JumpingEndState {
            self.context = self.context.update(JUMPING_FRAMES);

            if self.context.position.y >= FLOOR {
                JumpingEndState::Landing(self.land())
            } else {
                JumpingEndState::Jumping(self)
            }
        }

        pub fn land(self) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.reset_frame(),
                _state: Running {},
            }
        }
    }


    #[derive(Copy, Clone)]
    pub struct RedHatBoyState<S> {
        context: RedHatBoyContext,
        _state: S,
    }

    impl<S> RedHatBoyState<S> {
        pub fn context(&self) -> &RedHatBoyContext {
            &self.context
        }

        fn update_context(&mut self, frames: u8) {
            self.context = self.context.update(frames);
        }
    }


    #[derive(Copy, Clone)]
    pub struct RedHatBoyContext {
        pub frame: u8,
        pub position: Point,
        velocity: Point,
    }

    impl RedHatBoyContext {
        pub fn update(mut self, frame_count: u8) -> Self {
            self.velocity.y += GRAVITY;

            if self.frame < frame_count {
                self.frame += 1;
            } else {
                self.frame = 0;
            }

            self.position.x += self.velocity.x;
            self.position.y += self.velocity.y;

            if self.position.y > FLOOR {
                self.position.y = FLOOR;
            }

            self
        }

        fn reset_frame(mut self) -> Self {
            self.frame = 0;
            
            self
        }

        fn run_right(mut self) -> Self {
            self.velocity.x += RUNNING_SPEED;
            
            self
        }

        fn set_vertical_velocity(mut self, y: i16) -> Self {
            self.velocity.y = y;
            self
        }


    }//^-- impl 

}//^-- mod red_hat_boy_states

