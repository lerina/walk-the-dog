use std::{collections::HashMap, rc::Rc, sync::Mutex};
use crate::{ browser, engine::{self, Game, Point, Rect, Renderer, Sheet, KeyState},};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::Deserialize;
use web_sys::HtmlImageElement;

use self::red_hat_boy_states::*;

pub struct WalkTheDog {
    rhb: Option<RedHatBoy>,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog {
            rhb: None,
        }
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {

    async fn initialize(&self) -> Result<Box<dyn Game>> {
        let sheet: Option<Sheet> = 
                        browser::fetch_json("../resources/pix/rhb.json").await?.into_serde()?;        
        let image = Some(engine::load_image("../resources/pix/rhb.png").await?);
        
        Ok(Box::new(WalkTheDog { rhb: Some(RedHatBoy::new( 
                                    sheet.clone().ok_or_else(|| anyhow! ("No Sheet Present"))?,
                                    image.clone().ok_or_else(|| anyhow! ("No Image Present"))?,
                                 )),
        })) //^-- Ok
    }//^-- async fn initialize

    fn update(&mut self, keystate: &KeyState) {
        let mut velocity = Point { x: 0, y: 0 };

        //if keystate.is_pressed("ArrowDown") { velocity.y += 3; }
        if keystate.is_pressed("ArrowDown") {
            self.rhb.as_mut().unwrap().slide();
        }
        if keystate.is_pressed("ArrowUp") { velocity.y -= 3; }
        //if keystate.is_pressed("ArrowRight") { velocity.x += 3; }
        if keystate.is_pressed("ArrowRight") {
            velocity.x += 3;
            self.rhb.as_mut().unwrap().run_right();
        }
        if keystate.is_pressed("ArrowLeft") { velocity.x -= 3; }
        if keystate.is_pressed("ArrowLeft") { velocity.x -= 3; }
        
        self.rhb.as_mut().unwrap().update();
    }

    fn draw(&self, renderer: &Renderer) {
        renderer.clear( &Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 600.0,
                        height: 600.0,
        });


        self.rhb.as_ref().unwrap().draw(renderer);
    }//^-- draw()

}//^-- impl Game for WalkTheDog

//--------------------------------------------

struct RedHatBoy {
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
}//^-- impl RedHatBoy


//----------

#[derive(Copy, Clone)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    Running(RedHatBoyState<Running>),
    Sliding(RedHatBoyState<Sliding>),
}

pub enum Event {
    Run,
    Slide,
}


impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            (RedHatBoyStateMachine::Idle(state), Event::Run) => state.run().into(), 
            (RedHatBoyStateMachine::Running(state), Event::Slide) => state.slide().into(),
            _ => self,
        }
    }

    fn frame_name(&self) ->&str {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.frame_name(),
            RedHatBoyStateMachine::Running(state) => state.frame_name(),
            RedHatBoyStateMachine::Sliding(state) => state.frame_name(),
        }
    }


    fn context(&self) ->&RedHatBoyContext {
        match self {
            RedHatBoyStateMachine::Idle(state) =>&state.context(),
            RedHatBoyStateMachine::Running(state) =>&state.context(),
            RedHatBoyStateMachine::Sliding(state) => &state.context(),
        }
    }


    fn update(self) -> Self {
        match self {
            RedHatBoyStateMachine::Idle(mut state) => {
                state.update();
                RedHatBoyStateMachine::Idle(state)
            },
            RedHatBoyStateMachine::Running(mut state) => {
                state.update();
                RedHatBoyStateMachine::Running(state)
            },
            RedHatBoyStateMachine::Sliding(mut state) => {
                state.update();
                RedHatBoyStateMachine::Sliding(state)
            },
        }
    }
}//^-- impl


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



mod red_hat_boy_states {
    use crate::engine::Point;
    
    const FLOOR: i16 = 475;
    const IDLE_FRAME_NAME: &str = "Idle";
    const RUN_FRAME_NAME: &str = "Run";
    const IDLE_FRAMES: u8 = 29;
    const RUNNING_FRAMES: u8 = 23;
    const RUNNING_SPEED: i16 = 3;
    const SLIDING_FRAMES: u8 = 14;
    const SLIDING_FRAME_NAME: &str = "Slide";

    #[derive(Copy, Clone)]
    pub struct Idle;

    #[derive(Copy, Clone)]
    pub struct Running;

    #[derive(Copy, Clone)]
    pub struct Sliding;

    #[derive(Copy, Clone)]
    pub struct RedHatBoyState<S> {
        context: RedHatBoyContext,
        _state: S,
    }

    impl RedHatBoyState<Idle> {

        pub fn new() -> Self {
            RedHatBoyState {
                context: RedHatBoyContext {
                        frame: 0,
                        position: Point { x: 0, y: FLOOR },
                        velocity: Point { x: 0, y: 0 },
                },
                _state: Idle {},
            }//^-- RedHatBoyState
        }//^-- fn new

        pub fn run(self) -> RedHatBoyState<Running> {
            RedHatBoyState {
               context: self.context.reset_frame().run_right(),
                _state: Running {},
            }
        }

        pub fn frame_name(&self) -> &str {
            IDLE_FRAME_NAME
        }

        pub fn update(&mut self) {
            self.context = self.context.update(IDLE_FRAMES);
        }

    }//^-- impl RedHatBoyState<Idle>

    impl RedHatBoyState<Running> {
        pub fn frame_name(&self) -> &str {
            RUN_FRAME_NAME
        }

        pub fn update(&mut self) {
            self.context = self.context.update(RUNNING_FRAMES);
        }

        pub fn slide(self) -> RedHatBoyState<Sliding> {
            RedHatBoyState {
                context: self.context.reset_frame(),
                _state: Sliding {},
            }
        }
    }


    impl RedHatBoyState<Sliding> {
        pub fn frame_name(&self) -> &str {
            SLIDING_FRAME_NAME
        }

        pub fn update(&mut self) {
            self.context = self.context.update(SLIDING_FRAMES);
        }
    }

    impl<S> RedHatBoyState<S> {
        pub fn context(&self) -> &RedHatBoyContext {
            &self.context
        }
    }


    #[derive(Copy, Clone)]
    pub struct RedHatBoyContext {
        pub frame: u8,
        pub position: Point,
        pub velocity: Point,
    }

    impl RedHatBoyContext {

        pub fn update(mut self, frame_count: u8) -> Self {
            if self.frame < frame_count {
                self.frame += 1;
            } else {
                self.frame = 0;
            }
            self.position.x += self.velocity.x;
            self.position.y += self.velocity.y;
            
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


    }//^-- impl 

}//^-- mod red_hat_boy_states

