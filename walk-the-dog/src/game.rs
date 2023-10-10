use std::{collections::HashMap, rc::Rc, sync::Mutex};
use crate::{ browser, engine::{self, Game, Point, Rect, Renderer, Sheet, KeyState},};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::Deserialize;
use web_sys::HtmlImageElement;

use self::red_hat_boy_states::*;

pub struct WalkTheDog {
    image: Option<HtmlImageElement>,
    sheet: Option<Sheet>,
    frame: u8,
    position: Point,
    rhb: Option<RedHatBoy>,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog {
            image: None,
            sheet: None,
            frame: 0,
            position: Point {x: 0, y: 0},
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
        
        Ok(Box::new(WalkTheDog { image: image.clone(),
                                 sheet: sheet.clone(),
                                 frame: self.frame,
                                 position: self.position,
                                 rhb: Some(RedHatBoy::new( 
                                    sheet.clone().ok_or_else(|| anyhow! ("No Sheet Present"))?,
                                    image.clone().ok_or_else(|| anyhow! ("No Image Present"))?,
                                 )),
        })) //^-- Ok
    }//^-- async fn initialize

    fn update(&mut self, keystate: &KeyState) {
        let mut velocity = Point { x: 0, y: 0 };
        
        if keystate.is_pressed("ArrowDown") { velocity.y += 3; }
        if keystate.is_pressed("ArrowUp") { velocity.y -= 3; }
        if keystate.is_pressed("ArrowRight") { velocity.x += 3; }
        if keystate.is_pressed("ArrowLeft") { velocity.x -= 3; }
        if keystate.is_pressed("ArrowLeft") { velocity.x -= 3; }
        
        self.position.x += velocity.x;
        self.position.y += velocity.y;

        self.rhb.as_mut().unwrap().update();
    }

    fn draw(&self, renderer: &Renderer) {
        let current_sprite = (self.frame / 3) + 1;
        let frame_name = format!("Run ({}).png", current_sprite);
        let sprite = self.sheet.as_ref()
                               .and_then(|sheet| sheet.frames.get(&frame_name))
                               .expect("Cell not found");

        renderer.clear( &Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 600.0,
                        height: 600.0,
        });

        self.image.as_ref().map(|image| {
            renderer.draw_image(&self.image.as_ref().unwrap(),
                &Rect {  x: sprite.frame.x.into(),
                        y: sprite.frame.y.into(),
                        width: sprite.frame.w.into(),
                        height: sprite.frame.h.into(),
                },
                &Rect { x: self.position.x.into(),
                        y: self.position.y.into(),
                        width: sprite.frame.w.into(),
                        height: sprite.frame.h.into(),
                },
            );
        });


        self.rhb.as_ref().unwrap().draw(renderer);
    }//^-- draw()
}

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
}//^-- impl RedHatBoy

#[derive(Copy, Clone)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    Running(RedHatBoyState<Running>),
}

pub enum Event {
    Run,
}

impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            (RedHatBoyStateMachine::Idle(state), 
             Event::Run) => state.run().into(), 
            _ => self,
        }
    }

    fn frame_name(&self) ->&str {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.frame_name(),
            RedHatBoyStateMachine::Running(state) => state.frame_name(),
        }
    }

    fn context(&self) ->&RedHatBoyContext {
        match self {
            RedHatBoyStateMachine::Idle(state) =>&state.context(),
            RedHatBoyStateMachine::Running(state) =>&state.context(),
        }
    }

    fn update(self) -> Self {
        match self {
            RedHatBoyStateMachine::Idle(mut state) => {
                if state.context.frame < 29 {
                    state.context.frame += 1;
                } else {
                    state.context.frame = 0;
                }

                RedHatBoyStateMachine::Idle(state)
            }
            RedHatBoyStateMachine::Running(_) => self,
        }
    }
}//^-- impl


impl From<RedHatBoyState<Running>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Running>) -> Self {
        RedHatBoyStateMachine::Running(state)
    }
}

mod red_hat_boy_states {
    use crate::engine::Point;
    
    const FLOOR: i16 = 475;
    const IDLE_FRAME_NAME: &str = "Idle";
    const RUN_FRAME_NAME: &str = "Run";

    #[derive(Copy, Clone)]
    pub struct Idle;

    #[derive(Copy, Clone)]
    pub struct Running;


    #[derive(Copy, Clone)]
    pub struct RedHatBoyState<S> {
        pub context: RedHatBoyContext,
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
                context: self.context,
                _state: Running {},
            }
        }

        pub fn frame_name(&self) -> &str {
            IDLE_FRAME_NAME
        }

    }//^-- impl RedHatBoyState<Idle>

    impl RedHatBoyState<Running> {
        pub fn frame_name(&self) -> &str {
            RUN_FRAME_NAME
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

}//^-- mod red_hat_boy_states

