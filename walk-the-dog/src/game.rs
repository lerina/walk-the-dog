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
        //let sheet: Sheet = browser::fetch_json("../resources/pix/rhb.json").await?.into_serde()?;
        let sheet: Option<Sheet> = 
                        browser::fetch_json("../resources/pix/rhb.json").await?.into_serde()?;        
        let image = Some(engine::load_image("../resources/pix/rhb.png").await?);
        //let sheet = Some(sheet);

        //Ok(Box::new(WalkTheDog { image, sheet, frame: self.frame, position: self.position, }))
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
}


impl From<RedHatBoyState<Running>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Running>) -> Self {
        RedHatBoyStateMachine::Running(state)
    }
}

mod red_hat_boy_states {
    use crate::engine::Point;
    
    const FLOOR: i16 = 475;

    #[derive(Copy, Clone)]
    pub struct Idle;

    #[derive(Copy, Clone)]
    pub struct Running;


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
                context: self.context,
                _state: Running {},
            }
        }
    }

    #[derive(Copy, Clone)]
    pub struct RedHatBoyContext {
        frame: u8,
        position: Point,
        velocity: Point,
    }

}//^-- mod red_hat_boy_states

