use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
//use web_sys::console;
use rand::prelude::*;


// Note that if you import a JS function with Result you need 
// #[wasm_bindgen(catch)] to be annotated on the import 
// (unlike exported functions, which require no extra annotation). 
// This may not be necessary in the future though and it may work "as is"!.
// see: wasm-bindgen [Result Type](https://rustwasm.github.io/docs/wasm-bindgen/reference/types/result.html)
// [similar issue that gave the solution](https://github.com/rustwasm/wasm-bindgen/issues/2919)
// also see: wasm-bindgen [catch](https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-js-imports/catch.html)
// This is because we use catch on our Js side 
// const wasm = await init().catch(console.error);
#[wasm_bindgen(start, catch)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

//    console::log_1(&JsValue::from_str("Hello world!"));
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
                    .get_element_by_id("canvas")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .unwrap();

    let context = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()
                    .unwrap();

        /*
        context.move_to(300.0, 0.0); // top of triangle
        context.begin_path();
        context.line_to(0.0, 600.0); // bottom left of triangle
        context.line_to(600.0, 600.0); // bottom right of triangle
        context.line_to(300.0, 0.0); // back to top of triangle
        context.close_path();
        context.stroke();
        context.fill();
        */
        sierpinski(&context, 
                   [(300.0, 0.0), (0.0, 600.0), (600.0, 600.0)], 
                   (0, 255, 0), 
                   5);

    Ok(())
}

fn sierpinski(context: &web_sys::CanvasRenderingContext2d, 
              points: [(f64, f64); 3],
              color: (u8, u8, u8), 
              depth: u8) {

        //draw_triangle(&context, [(300.0, 0.0), (0.0, 600.0), (600.0, 600.0)]);
        draw_triangle(&context, points, color);
        
        let depth = depth - 1;
        let [top, left, right] = points;
        
        if depth > 0 {
            let mut rng = thread_rng();

            let next_color = (
                rng.gen_range(0..255),
                rng.gen_range(0..255),
                rng.gen_range(0..255),
            );
        /*//01.
        draw_triangle(&context, [(300.0, 0.0), (150.00, 300.0), (450.0, 300.0)]);
        draw_triangle(&context, [(150.0, 300.0), (0.0, 600.0), (300.0, 600.0)]);
        draw_triangle(&context, [(450.0, 300.0), (300.0, 600.0), (600.0, 600.0)]);
        */
        /* //02.
            sierpinski(&context, 
                        [(300.0, 0.0), (150.00, 300.0), (450.0, 300.0)], 
                        depth,);
            sierpinski(&context,
                        [(150.0, 300.0), (0.0, 600.0), (300.0, 600.0)],
                        depth,);
            sierpinski(&context,
                        [(450.0, 300.0), (300.0, 600.0), (600.0, 600.0)],
                        depth,);
         */
         /* //03.
            let left_middle = ((top.0 + left.0) / 2.0, (top.1 + left.1) / 2.0);
            let right_middle = ((top.0 + right.0) / 2.0, (top.1 + right.1) / 2.0);
            let bottom_middle = (top.0, right.1);
         */
            let left_middle = midpoint(top, left);
            let right_middle = midpoint(top, right);
            let bottom_middle = midpoint(left, right);
   
            sierpinski(&context, 
                       [top, left_middle, right_middle], next_color, depth);
            sierpinski(&context, 
                       [left_middle, left, bottom_middle], next_color, depth);
            sierpinski(&context, 
                       [right_middle, bottom_middle, right], next_color, depth);



        }//^-- if depth
}

fn midpoint(point_1: (f64, f64), point_2: (f64, f64)) -> (f64, f64) {
    ((point_1.0 + point_2.0) / 2.0, 
     (point_1.1 + point_2.1)/ 2.0)
}

fn draw_triangle(context: &web_sys::CanvasRenderingContext2d,
                 points: [(f64, f64); 3], 
                 color: (u8, u8, u8),) {
    let [top, left, right] = points;

    // context.fillStyle property takes DOMString in JavaScript, 
    // so we'll need to do a conversion.
    let color_str = format!("rgb({}, {}, {})", color.0, color.1, color.2);
    context.set_fill_style(&wasm_bindgen::JsValue::from_str(&color_str));
    /*NOTE: 
generally, JavaScript properties are just public and you set them, but web-sys generates getter and setter functions. 
These generated functions frequently take JsValue objects, which represent an object owned by JavaScript. 
Fortunately, wasm_bindgen has factory functions for these, so we can create them easily and use the compiler as our guide.    
    */

    context.move_to(top.0, top.1);
    context.begin_path();
    context.line_to(left.0, left.1);
    context.line_to(right.0, right.1);
    context.line_to(top.0, top.1);
    context.close_path();
    context.stroke();
    // Now we have color we can fill
    context.fill();
}
