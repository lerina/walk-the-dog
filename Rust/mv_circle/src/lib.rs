/*
wasm setup rs/js/html
    - rs
    - html
    - js

display dot
    - show canvas with bg
    - place dot mid height, at the left

mv dot straight line
    - update dot x pos

mv dot with kb arrows
    - capture kb  press
    - increase dot x pos if right arrow kb press-release

mv dot in sine wave
    update dot x,y pos though sinewave function
*/
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use rand::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

//-----------------------------------------------------
#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Circle {
    pub center_x: f32, 
    pub center_y: f32, 
    pub radius: f32,
    pub start_angle: f32, 
    pub end_angle: f32,
}

#[wasm_bindgen]
impl Circle {
    #[wasm_bindgen(constructor)]
    pub fn new( center_x: f32, 
                center_y: f32, 
                radius: f32, 
                start_angle: f32, 
                end_angle: f32,) -> Circle {
    //  center_x, center_y, radius, 0, 2 * Math.PI, false
        Circle {
            center_x, center_y, radius,
            start_angle: 0.0, 
            end_angle: 2.0 * std::f32::consts::PI,
        }
    }

    pub fn rnd_new() -> Circle {
        let mut rng = thread_rng();
        let radius = 5.0 * rng.gen::<f32>() * 40.0;
        let center_x: f32 = rng.gen_range((0.0 + radius)..(600.0 - radius));
        let center_y: f32 = rng.gen_range((0.0 + radius)..(600.0 - radius));
        let start_angle: f32 = 0.0;
        let end_angle: f32 = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
        //let counter_clockwise: bool = rng.gen::<bool>();

        Circle::new( center_x,
                    center_y,
                    radius,
                    start_angle,
                    end_angle,
                    )
    }
    
    pub fn mv_right(&mut self, v: f32, max:f32) {
        if self.center_x + v <= max - self.radius {
            self.center_x += v;
        }
    }
    pub fn mv_left(&mut self, v: f32, max:f32) {
        if self.center_x + v >=self.radius {
            self.center_x += v;
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct World {
    pub width: f32,
    pub height: f32,
    pub circle: Circle,
    pub circle_color: String,
}

#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new(width:f32, height:f32) -> World {


        let circle = Circle::rnd_new();
        let circle_color = "rgba(177, 0, 129, .1)".to_string();
        World { width, height, circle, circle_color }    
    }

}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    


    log("Rust wasm initialized!");

    Ok(())
}

//--------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
