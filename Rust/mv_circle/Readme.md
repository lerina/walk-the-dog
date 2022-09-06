Q: Is it possible to have color as a Circle attribute?

For now I can't  compose World with Circle using 
`#[wasm_bindgen(getter_with_clone)]`

So this works 

```rust
#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Circle {
    pub center_x: f32, 
    pub center_y: f32, 
    pub radius: f32,
    pub start_angle: f32, 
    pub end_angle: f32,
}

#[wasm_bindgen(getter_with_clone)]
pub struct World {
    pub width: f32,
    pub height: f32,
    pub circle: Circle,
    pub circle_color: String,
}
```
but this does not compile

```rust
#[wasm_bindgen(getter_with_clone)]
pub struct Circle {
    pub center_x: f32, 
    pub center_y: f32, 
    pub radius: f32,
    pub start_angle: f32, 
    pub end_angle: f32,
    pub color: String,
}

#[wasm_bindgen(getter_with_clone)]
pub struct World {
    pub width: f32,
    pub height: f32,
    pub circle: Circle,
}
```
