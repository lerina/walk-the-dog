use std::rc::Rc;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use web_sys::HtmlImageElement;

use self::red_hat_boy_states::*;
use crate::{
    browser,
    engine::{self, Cell, Game, Image, KeyState, Point, Rect, Renderer, Sheet, SpriteSheet},
};

const HEIGHT: i16 = 600;

const FIRST_PLATFORM: i16 = 200;
const LOW_PLATFORM: i16 = 400;


pub struct Barrier {
    image: Image,
}

impl Barrier {
    pub fn new(image: Image) -> Self {
        Barrier { image }
    }
}


impl Obstacle for Barrier {
    //fn check_intersection(&self, boy: &mut RedHatBoy) { todo!() }
    fn check_intersection(&self, boy: &mut RedHatBoy) {
        if boy.bounding_box().intersects(self.image.bounding_box()) {
            boy.knock_out()
        }
    }

    fn draw(&self, renderer: &Renderer) {
        self.image.draw(renderer);
    }

    fn move_horizontally(&mut self, x: i16) {
        self.image.move_horizontally(x);
    }
    fn right(&self) -> i16 {
        self.image.right()
    }
}//^-- impl Obstacle for Barrier


pub trait Obstacle {
    fn check_intersection(&self, boy: &mut RedHatBoy);
    fn draw(&self, renderer: &Renderer);
    //fn draw_rect(&self, renderer: &Renderer);
    fn move_horizontally(&mut self, x: i16);
    fn right(&self) -> i16;
}

/*
struct Platform {
    sheet: Sheet,
    image: HtmlImageElement,
    position: Point,
}
*/
/*
pub struct Platform {
    //sheet: SpriteSheet,
    sheet: Rc<SpriteSheet>,
    position: Point,
}
*/
pub struct Platform {
    sheet: Rc<SpriteSheet>,
    bounding_boxes: Vec<Rect>,
    sprites: Vec<Cell>,
    position: Point,
}

impl Platform {
/*
    //fn new(sheet: Sheet, image: HtmlImageElement, position: Point) -> Self {    
    fn new(sheet: SpriteSheet, image: HtmlImageElement, position: Point) -> Self {
        Platform {
            /*sheet,
            image,
            */
            sheet,
            position,
        }
    }//^-- new
*/
/*    pub fn new(sheet: Rc<SpriteSheet>, position: Point) -> Self {
        Platform { sheet, position }
    }
*/

    pub fn new( sheet: Rc<SpriteSheet>, position: Point,
                sprite_names: &[&str], bounding_boxes: &[Rect],) -> Self {

        let sprites = sprite_names
                            .iter()
                            .filter_map(|sprite_name|
                            sheet.cell(sprite_name).cloned())
                            .collect();
        let bounding_boxes = bounding_boxes
                                .iter()
                                .map(|bounding_box| {
                                    Rect::new_from_x_y(
                                        bounding_box.x() + position.x,
                                        bounding_box.y() + position.y,
                                        bounding_box.width,
                                        bounding_box.height,
                                    )
                                })
                                .collect();

        Platform {
            sheet,
            position,
            sprites,
            bounding_boxes,
        }
    }//^-- fn new



/* //No longer used using mutiple platform
    fn destination_box(&self) ->Rect {
        let platform = self
                        .sheet
                        .frames
                        .get("13.png")
                        .expect("13.png does not exist");
        
        Rect {
            position: Point{ x: self.position.x.into(), y: self.position.y.into()},                       
            width: (platform.frame.w * 3).into(),
            height: platform.frame.h.into(),
        }
    }//^-- destination_box
*/

/*
    fn bounding_boxes(&self) -> Vec<Rect> {
        const X_OFFSET: i16 = 60; 
        const END_HEIGHT: i16 = 54; 
        let destination_box = self.destination_box();

        let bounding_box_one = Rect {
            position: Point {
                x: destination_box.x(),
                y: destination_box.y()},
            width: X_OFFSET,
            height: END_HEIGHT,
        };
        let bounding_box_two = Rect {
            position: Point { 
                x: destination_box.x() + X_OFFSET,
                y: destination_box.y()
            },
            width: destination_box.width - (X_OFFSET * 2),
            height: destination_box.height,
        };

        let bounding_box_three = Rect {
            position: Point {
                x: destination_box.x() + destination_box.width - X_OFFSET,
                y: destination_box.y()
            },
            width: X_OFFSET,
            height: END_HEIGHT,
        };

        vec![bounding_box_one, bounding_box_two, bounding_box_three]
    }//^-- fn bounding_boxes
*/
    fn bounding_boxes(&self) -> &Vec<Rect> {
        &self.bounding_boxes
    }

    fn draw_rect(&self, renderer: &Renderer){
        for bounding_box in self.bounding_boxes() {
            renderer.draw_rect(&bounding_box);
        }
    }

}

impl Obstacle for Platform {

/*    
    fn draw(&self, renderer: &Renderer) {
        let platform = self
                        .sheet
                        .frames
                        .get("13.png")
                        .expect("13.png does not exist");
        
        renderer.draw_image( &self.image,
                             &Rect {
                                     position: Point {
                                     x: platform.frame.x.into(),
                                     y: platform.frame.y.into(),
                                 },
                                 width: (platform.frame.w * 3).into(),
                                 height: platform.frame.h.into(),
                             },
                             &self.destination_box(),
                           );
    }//^-- draw
*/
    fn draw(&self, renderer: &Renderer) {
        let mut x = 0;
        self.sprites.iter().for_each(|sprite| {
            self.sheet.draw(
                renderer,
                &Rect::new_from_x_y(
                    sprite.frame.x,
                    sprite.frame.y,
                    sprite.frame.w,
                    sprite.frame.h,
                ),
                // Just use position and the standard
                // widths in the tileset
                &Rect::new_from_x_y(
                    self.position.x + x,
                    self.position.y,
                    sprite.frame.w,
                    sprite.frame.h,
                ),
            );
            x += sprite.frame.w;
        });
    }//^-- fn draw
/*
    fn move_horizontally(&mut self, x: i16) {
        self.position.x += x;
    }
*/
    fn move_horizontally(&mut self, x: i16) {
        self.position.x += x;
        self.bounding_boxes.iter_mut()
                           .for_each(|bounding_box| {
                                bounding_box.set_x(bounding_box.position.x + x);
                            });
    }

    fn check_intersection(&self, boy: &mut RedHatBoy) {
        if let Some(box_to_land_on) = self
                .bounding_boxes()
                .iter()
                .find(|&bounding_box| boy.bounding_box()
                .intersects(bounding_box))
        {
            if boy.velocity_y() > 0 && boy.pos_y() < self.position.y {
                    boy.land_on(box_to_land_on.y());
            } else {
                boy.knock_out();
            }
        }
    }//^-- check_intersection

    fn right(&self) -> i16 {
        self.bounding_boxes()
            .last()
            .unwrap_or(&Rect::default())
            .right()
    }

}//^-- impl Obstacle

pub struct RedHatBoy {
    state_machine: RedHatBoyStateMachine,
    sprite_sheet: Sheet,
    image: HtmlImageElement,
}

impl RedHatBoy {
    fn new(sprite_sheet: Sheet, image: HtmlImageElement) -> Self {
        RedHatBoy {
            state_machine: RedHatBoyStateMachine::Idle(RedHatBoyState::new()),
            sprite_sheet,
            image,
        }
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

    fn update(&mut self) {
        self.state_machine = self.state_machine.update();
    }

    fn frame_name(&self) -> String {
        format!(
            "{} ({}).png",
            self.state_machine.frame_name(),
            (self.state_machine.context().frame / 3) + 1
        )
    }

    fn current_sprite(&self) -> Option<&Cell> {
        self
            .sprite_sheet
            .frames
            .get(&self.frame_name())
    }

    fn bounding_box(&self) -> Rect {
        const X_OFFSET: i16 = 18;     
        const Y_OFFSET: i16 = 14;     
        const WIDTH_OFFSET: i16 = 28; 
        let mut bounding_box = self.destination_box();
        bounding_box.position.x += X_OFFSET;
        bounding_box.width -= WIDTH_OFFSET;
        bounding_box.position.y += Y_OFFSET;
        bounding_box.height -= Y_OFFSET;
        bounding_box
    }

    fn destination_box(&self) -> Rect {
        let sprite = self.current_sprite().expect("Cell not found");

        Rect {
            position: Point {
                x: (self.state_machine.context().position.x + sprite.sprite_source_size.x)
                .into(),
                y: (self.state_machine.context().position.y + sprite.sprite_source_size.y)
                .into(),
            },
            width: sprite.frame.w.into(),
            height: sprite.frame.h.into(),
        }
    }


    fn draw_rect(&self, renderer: &Renderer){
        renderer.draw_rect(&self.bounding_box());
    }

    fn draw(&self, renderer: &Renderer) {
        let sprite = self.current_sprite().expect("Cell not found");
        

        renderer.draw_image(
            &self.image,
            &Rect {
                position: Point {
                    x: sprite.frame.x.into(),
                    y: sprite.frame.y.into(),
                },
                width: sprite.frame.w.into(),
                height: sprite.frame.h.into(),
            },
            &self.destination_box(),
        );
    }//^-- fn draw

    fn knock_out(&mut self) {
        self.state_machine = self.state_machine.transition(Event::KnockOut);
    }

    fn land_on(&mut self, position: i16) { // f32) {
        self.state_machine = self.state_machine.transition(Event::Land(position));
    }

    fn pos_y(&self) -> i16 {
        self.state_machine.context().position.y
    }

    fn velocity_y(&self) -> i16 {
        self.state_machine.context().velocity.y
    }

    fn walking_speed(&self) -> i16 {
        self.state_machine.context().velocity.x
    }
}//^-- impl RedHatBoy 

#[derive(Copy, Clone)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    Running(RedHatBoyState<Running>),
    Sliding(RedHatBoyState<Sliding>),
    Jumping(RedHatBoyState<Jumping>),
    Falling(RedHatBoyState<Falling>),
    KnockedOut(RedHatBoyState<KnockedOut>),
}

pub enum Event {
    Run,
    Slide,
    Update,
    Jump,
    KnockOut,
    Land(i16),  //  f32),
}

impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            (RedHatBoyStateMachine::Idle(state),    Event::Run) => state.run().into(),
            (RedHatBoyStateMachine::Running(state), Event::Jump) => state.jump().into(),
            (RedHatBoyStateMachine::Running(state), Event::Slide) => state.slide().into(),

            (RedHatBoyStateMachine::Idle(state),    Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Running(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Jumping(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Sliding(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Falling(state), Event::Update) => state.update().into(),

            (RedHatBoyStateMachine::Running(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Jumping(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Sliding(state), Event::KnockOut) => state.knock_out().into(),

            (RedHatBoyStateMachine::Jumping(state), Event::Land(position)) => state.land_on(position).into(),
            (RedHatBoyStateMachine::Running(state), Event::Land(position)) => state.land_on(position).into(),
            (RedHatBoyStateMachine::Sliding(state), Event::Land(position)) => state.land_on(position).into(),
            _ => self,
        }
    }

    fn frame_name(&self) -> &str {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.frame_name(),
            RedHatBoyStateMachine::Running(state) => state.frame_name(),
            RedHatBoyStateMachine::Jumping(state) => state.frame_name(),
            RedHatBoyStateMachine::Sliding(state) => state.frame_name(),
            RedHatBoyStateMachine::Falling(state) => state.frame_name(),
            RedHatBoyStateMachine::KnockedOut(state) => state.frame_name(),
        }
    }

    fn context(&self) -> &RedHatBoyContext {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.context(),
            RedHatBoyStateMachine::Running(state) => state.context(),
            RedHatBoyStateMachine::Jumping(state) => state.context(),
            RedHatBoyStateMachine::Sliding(state) => state.context(),
            RedHatBoyStateMachine::Falling(state) => state.context(),
            RedHatBoyStateMachine::KnockedOut(state) => state.context(),
        }
    }

    fn update(self) -> Self {
        self.transition(Event::Update)
    }
}

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

impl From<RedHatBoyState<Falling>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Falling>) -> Self {
        RedHatBoyStateMachine::Falling(state)
    }
}

impl From<RedHatBoyState<KnockedOut>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<KnockedOut>) -> Self {
        RedHatBoyStateMachine::KnockedOut(state)
    }
}

impl From<SlidingEndState> for RedHatBoyStateMachine {
    fn from(state: SlidingEndState) -> Self {
        match state {
            SlidingEndState::Sliding(sliding) => sliding.into(),
            SlidingEndState::Running(running) => running.into(),
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

impl From<FallingEndState> for RedHatBoyStateMachine {
    fn from(state: FallingEndState) -> Self {
        match state {
            FallingEndState::Falling(falling) => falling.into(),
            FallingEndState::KnockedOut(knocked_out) => knocked_out.into(),
        }
    }
}

mod red_hat_boy_states {
    use crate::engine::Point;
    use super::HEIGHT;

    //const FLOOR: i16 = 475;
    const FLOOR: i16 = 479;
    const PLAYER_HEIGHT: i16 = HEIGHT - FLOOR;
    const STARTING_POINT: i16 = -20;
    const IDLE_FRAMES: u8 = 29;
    const RUNNING_FRAMES: u8 = 23;
    const JUMPING_FRAMES: u8 = 35;
    const SLIDING_FRAMES: u8 = 14;
    const FALLING_FRAMES: u8 = 29; // 10 'Dead' frames in the sheet, * 3 - 1.
    
    const IDLE_FRAME_NAME: &str = "Idle";
    const RUN_FRAME_NAME: &str = "Run";
    const SLIDING_FRAME_NAME: &str = "Slide";
    const JUMPING_FRAME_NAME: &str = "Jump";
    const FALLING_FRAME_NAME: &str = "Dead";

    const RUNNING_SPEED: i16 = 3;    
    const JUMP_SPEED: i16 = -25;
    const GRAVITY: i16 = 1;
    const TERMINAL_VELOCITY: i16 = 20;


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
    pub struct Idle;

    impl RedHatBoyState<Idle> {
        pub fn new() -> Self {
            RedHatBoyState {
                context: RedHatBoyContext {
                    frame: 0,
                    position: Point { x: STARTING_POINT, y: FLOOR, },
                    velocity: Point { x: 0, y: 0 },
                },
                _state: Idle {},
            }
        }

        pub fn frame_name(&self) -> &str {
            IDLE_FRAME_NAME
        }

        pub fn update(mut self) -> RedHatBoyState<Idle> {
            self.update_context(IDLE_FRAMES);
            self
        }

        pub fn run(self) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.reset_frame().run_right(),
                _state: Running {},
            }
        }
    }

    #[derive(Copy, Clone)]
    pub struct Running;

    impl RedHatBoyState<Running> {
        pub fn frame_name(&self) -> &str {
            RUN_FRAME_NAME
        }

        pub fn update(mut self) -> RedHatBoyState<Running> {
            self.update_context(RUNNING_FRAMES);
            self
        }

        pub fn jump(self) -> RedHatBoyState<Jumping> {
            RedHatBoyState {
                context: self.context.reset_frame().set_vertical_velocity(JUMP_SPEED),
                _state: Jumping {},
            }
        }

        pub fn slide(self) -> RedHatBoyState<Sliding> {
            RedHatBoyState {
                context: self.context.reset_frame(),
                _state: Sliding {},
            }
        }
        pub fn knock_out(self) -> RedHatBoyState<Falling> {
            RedHatBoyState { context: self.context.reset_frame().stop(),
                             _state: Falling {},
            }
        }
        //pub fn land_on(self, position: f32) -> RedHatBoyState<Running> {
        pub fn land_on(self, position: i16) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.set_on(position), // as i16),
                _state: Running {},
            }
        }
    }//^-- impl RedHatBoyState<Running>

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
            self.update_context(JUMPING_FRAMES);

            if self.context.position.y >= FLOOR {
                //JumpingEndState::Landing(self.land())
                JumpingEndState::Landing(self.land_on(HEIGHT.into()))
            } else {
                JumpingEndState::Jumping(self)
            }
        }

        //pub fn land_on(self, position: f32) -> RedHatBoyState<Running> {
        pub fn land_on(self, position: i16) -> RedHatBoyState<Running> {
                RedHatBoyState {
                    context: self.context.reset_frame().set_on(position), // as i16),
                    _state: Running,
                }
            
        }//^-- fn land_on


        pub fn knock_out(self) -> RedHatBoyState<Falling> {
            RedHatBoyState { context: self.context.reset_frame().stop(),
                             _state: Falling {},
            }
        }
    }

    #[derive(Copy, Clone)]
    pub struct Sliding;

    pub enum SlidingEndState {
        Sliding(RedHatBoyState<Sliding>),
        Running(RedHatBoyState<Running>),
    }

    impl RedHatBoyState<Sliding> {
        pub fn frame_name(&self) -> &str {
            SLIDING_FRAME_NAME
        }

        pub fn update(mut self) -> SlidingEndState {
            self.update_context(SLIDING_FRAMES);

            if self.context.frame >= SLIDING_FRAMES {
                SlidingEndState::Running(self.stand())
            } else {
                SlidingEndState::Sliding(self)
            }
        }

        pub fn stand(self) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.reset_frame(),
                _state: Running {},
            }
        }

        pub fn knock_out(self) -> RedHatBoyState<Falling> {
            RedHatBoyState { context: self.context.reset_frame().stop(),
                             _state: Falling {},
            }
        }

        //pub fn land_on(self, position: f32) -> RedHatBoyState<Sliding> {
        pub fn land_on(self, position: i16) -> RedHatBoyState<Sliding> {
            RedHatBoyState {
                context: self.context.set_on(position), // as i16),
                _state: Sliding {},
            }
        }
    }



   #[derive(Copy, Clone)]
   pub struct KnockedOut;

   impl RedHatBoyState<KnockedOut> {
        pub fn frame_name(&self) -> &str {
            FALLING_FRAME_NAME
        }
    }

    #[derive(Copy, Clone)]
    pub struct Falling;


    pub enum FallingEndState {
        Falling(RedHatBoyState<Falling>),
        KnockedOut(RedHatBoyState<KnockedOut>),
    }

    impl RedHatBoyState<Falling> {
        pub fn frame_name(&self) -> &str {
            FALLING_FRAME_NAME
        }

        pub fn knock_out(self) -> RedHatBoyState<KnockedOut> {
            RedHatBoyState {
                context: self.context,
                _state: KnockedOut {},
            }
        }
        
        pub fn update(mut self) -> FallingEndState {
            self.update_context(FALLING_FRAMES);
            if self.context.frame >= FALLING_FRAMES {
                FallingEndState::KnockedOut(self.knock_out())
            } else {
                FallingEndState::Falling(self)
            }
        }
 
    }//^-- impl RedHatBoyState<Falling>



    #[derive(Copy, Clone)]
    pub struct RedHatBoyContext {
        pub frame: u8,
        pub position: Point,
        pub velocity: Point,
    }

    impl RedHatBoyContext {
        pub fn update(mut self, frame_count: u8) -> Self {
            //self.velocity.y += GRAVITY;
            if self.velocity.y < TERMINAL_VELOCITY {
                self.velocity.y += GRAVITY;
            }

            if self.frame < frame_count {
                self.frame += 1;
            } else {
                self.frame = 0;
            }
            
             // scrolling background instead  
            //self.position.x += self.velocity.x;
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

        fn set_vertical_velocity(mut self, y: i16) -> Self {
            self.velocity.y = y;
            self
        }

        fn run_right(mut self) -> Self {
            self.velocity.x += RUNNING_SPEED;
            self
        }

        fn stop(mut self) -> Self {
            self.velocity.x = 0;
            self
        }

        fn set_on(mut self, position: i16) -> Self {
            let position = position - PLAYER_HEIGHT;
            self.position.y = position;
            self
        }
    }
}

/*
pub struct Walk {
    boy: RedHatBoy,
    backgrounds: [Image; 2],
    stone: Image,
    platform: Box<dyn Obstacle>, //platform: Platform,
}
*/
pub struct Walk {
    obstacle_sheet: Rc<SpriteSheet>,
    boy: RedHatBoy,
    backgrounds: [Image; 2],
    obstacles: Vec<Box<dyn Obstacle>>,
}

impl Walk {
    fn velocity(&self) -> i16 {
        -self.boy.walking_speed()
    }
}

pub enum WalkTheDog {
    Loading,
    Loaded(Walk),
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
                let background = engine::load_image("../resources/pix/BG.png").await?;
                let stone = engine::load_image("../resources/pix/Stone.png").await?;
                // change of name
                // let platform_sheet = browser::fetch_json("../resources/pix/tiles.json").await?;
                let tiles = browser::fetch_json("../resources/pix/tiles.json").await?;

                let sprite_sheet = Rc::new(SpriteSheet::new(
                                    tiles.into_serde::<Sheet>()?,
                                    engine::load_image("tiles.png").await?,
                                   ));
                
                /*
                let platform = Platform::new(  
                                   SpriteSheet::new(platform_sheet.into_serde::<Sheet>()?,
                                                    engine::load_image("../resources/pix/tiles.png").await?, 
                                                  ),
                                    Point { x: 200, y: 400 },
                               );        
                */
                /*
                let platform = Platform::new(
                                    sprite_sheet.clone(),
                                    Point {
                                        x: FIRST_PLATFORM,
                                        y: LOW_PLATFORM,
                                    },
                                );
                */
                let platform = Platform::new(
                                sprite_sheet.clone(),
                                Point {
                                    x: FIRST_PLATFORM,
                                    y: LOW_PLATFORM,
                                },
                                &["13.png", "14.png", "15.png"],
                                &[
                                    Rect::new_from_x_y(0, 0, 60, 54),
                                    Rect::new_from_x_y(60, 0, 384 - (60 * 2), 93),
                                    Rect::new_from_x_y(384 - 60, 0, 60, 54),
                                ],
                );
                let background_width = background.width() as i16;
                let backgrounds = [ Image::new( background.clone(), Point { x: 0, y: 0 }),
                                    Image::new( background, Point { x: background_width, y: 0,},),
                                  ];


                let obstacles = vec![ Box::new(Barrier::new(
                                                 Image::new( stone, Point { x: 150, y: 546 }))),
                                      Box::new(platform),
                                            ];

                /*
                let walk = Walk {   boy: rhb, 
                                    backgrounds: backgrounds,
                                    obstacles: obstacles,
                                };                
                */
                let walk = Walk {   boy: rhb, 
                                    backgrounds: backgrounds,
                                    obstacles: obstacles,
                                    obstacle_sheet: sprite_sheet, 
                                };

                Ok(Box::new(WalkTheDog::Loaded(walk)))
            },
            WalkTheDog::Loaded(_) => Err(anyhow!("Error: Game is already initialized")),
        }
    }//^-- async fn initialize

    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(walk) = self {
            if keystate.is_pressed("ArrowRight") {
                walk.boy.run_right();
            }

            if keystate.is_pressed("Space") {
                walk.boy.jump();
            }

            if keystate.is_pressed("ArrowDown") {
                walk.boy.slide();
            }

            walk.boy.update();

            let velocity = walk.velocity();

            walk.platform.position.x += walk.velocity();
            walk.platform.move_horizontally(velocity); //walk.velocity());
            walk.stone.move_horizontally(velocity);
            
 /*
            // check_intersection
            for bounding_box in &walk.platform.bounding_boxes() {
                if walk.boy.bounding_box().intersects(bounding_box) {
                    if walk.boy.velocity_y() > 0 && walk.boy.pos_y() < walk.platform.position.y {
                        walk.boy.land_on(bounding_box.y);
                    } else {
                        walk.boy.knock_out();
                    }
                }
            }

            //// check_intersection comment no longer needed
            //walk.platform.check_intersection(&mut walk.boy);   

            // knock_out
            if walk.boy
                   .destination_box()
                   .intersects(walk.stone.bounding_box())
            {
                walk.boy.knock_out();
            }
*/
            // background states
            let [first_background, second_background] = &mut walk.backgrounds;
            first_background.move_horizontally(velocity);
            second_background.move_horizontally(velocity);

            if first_background.right() < 0 {
                first_background.set_x(
                second_background.right());
            }
            if second_background.right() < 0 {
                second_background.set_x(
                first_background.right());
            }

            
            walk.obstacles.iter_mut().for_each(|obstacle| {
                obstacle.move_horizontally(velocity);
                obstacle.check_intersection(&mut walk.boy);
            });

        }//^-- if let
    }//^-- fn update

    fn draw(&self, renderer: &Renderer) {
        renderer.clear(&Rect {
            x: 0, 
            y: 0, 
            width: 600, 
            height: 600,
        });

        if let WalkTheDog::Loaded(walk) = self {
            
            walk.backgrounds.iter().for_each(|background| {
                background.draw(renderer);
            });
            walk.boy.draw(renderer);
            walk.boy.draw_rect(renderer);
            
            walk.stone.draw_rect(renderer);
            
            walk.platform.draw_rect(renderer);

            // removing an obstacle from the obstacles
            // Vec when they go off screen            
            walk.obstacles.retain(|obstacle|
                obstacle.right() > 0);

            // move and collide with the obstacles
            walk.obstacles.iter().for_each(|obstacle| {
                obstacle.draw(renderer);
            });
            
        }
    }
}

