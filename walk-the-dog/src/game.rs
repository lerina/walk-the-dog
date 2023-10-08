use std::{collections::HashMap, rc::Rc, sync::Mutex};
use crate::{ browser, engine::{self, Game, Point, Rect, Renderer, Sheet, KeyState},};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use web_sys::HtmlImageElement;


pub struct WalkTheDog {
    image: Option<HtmlImageElement>,
    sheet: Option<Sheet>,
    frame: u8,
    position: Point,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog {
            image: None,
            sheet: None,
            frame: 0,
            position: Point {x: 0, y: 0},
        }
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        let sheet: Sheet = browser::fetch_json("../resources/pix/rhb.json").await?.into_serde()?;
        let image = Some(engine::load_image("../resources/pix/rhb.png").await?);
        let sheet = Some(sheet);

        Ok(Box::new(WalkTheDog { image, sheet, frame: self.frame, position: self.position, }))
    }

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
