## Refactoring for endless running

### A more useful Rect

The `Rect` implementation only contains the intersects method, 
but there are two very useful methods it could use: `right` and `bottom`. 
If you look at the method we just wrote on Image, 
you will see that it's a natural fit for a `right` function. 

Let's go ahead and add it to Rect :


```rust
// src/engine.rs

/*
impl Rect {
    pub fn intersects(&self, rect: &Rect) -> bool {
        self.x < (rect.x + rect.width)
        && self.x + self.width > rect.x
        && self.y < (rect.y + rect.height)
        && self.y + self.height > rect.y
    }
}
*/

impl Rect {
    pub fn intersects(&self, rect: &Rect) -> bool {
        self.x < rect.right()
        && self.right() > rect.x
        && self.y < rect.bottom()
        && self.bottom() > rect.y
    }
    pub fn right(&self) -> i16 {
        self.x + self.width
    }
    pub fn bottom(&self) -> i16 {
        self.y + self.height
    }
}//^-- impl Rect

```
Adding the `right` and `bottom` methods will prevent that addition logic from getting
smeared across the game logic. We've also refactored intersects to use these new methods. 

Now, let's go back to the `Image` code we just wrote and update it to use the new `right` method, 
as shown here:



```rust
// src/engine.rs

impl Image {
    ...
    /*
    pub fn right(&self) -> i16 {
        (self.bounding_box.x + self.bounding_box.width) as i16
    }
    */

    pub fn right(&self) -> i16 {
        self.bounding_box.right()
    }
}
```

While we're in `Image`, let's deal with the duplication of `position` and `bounding_box`.

### Setting Rect's position

An image containing a `bounding_box` **Rect** and a `position` **Point** is an accident 
that occurred due to our code evolving. So, the question is, which one do we want to keep?

We could always keep `bounding_box` for the image, which would mean constructing a Point 
every time we draw because we need that for the `draw_entire_element` call.

We could also create a `Dimens` structure that just has `width` and `height`, 
and construct a `Rect` every time we need it on the update. 

While I doubt that the cost of creating those objects is going to be noticeable, 
the fact that it's on every frame is bothersome.

What we'll do instead is give `Rect` a `position` field – after all, 
that's what the `x` and `y` coordinates of `Rect` are. 
This is a seemingly minor change but with far-reaching implications 
because we constantly initialize `Rect` with `x` and `y`. 

Fortunately, we can use the compiler to make this simpler for us. 
We'll start by changing `Rect` to hold a `position` field, instead of `x` and `y`:



```rust
// src/engine.rs

/*
pub struct Rect {
    pub x: i16,
    pub y: i16,
    pub width: i16,
    pub height: i16,
}
*/

pub struct Rect {
    pub position: Point,
    pub width: i16,
    pub height: i16,
}

```
Adding `position` is going to cause compiler errors all over the place, as expected. 

We know that we frequently want to both access the `x` and `y` values 
and create a `Rect` using `x` and `y`, so to make it easier to work with, 
we'll add two **factory** methods for `Rect`, as shown here:


```rust
// src/engine.rs

impl Rect {
    pub fn new(position: Point, width: i16, height: i16) -> Self {
        Rect {
            position,
            width,
            height,
        }
    }

    pub fn new_from_x_y(x: i16, y: i16, width: i16, height: i16) -> Self {
        Rect::new(Point { x, y }, width, height)
    }

    pub fn intersects(&self, rect: &Rect) -> bool {
    ...
...

```

Now, when we fix Rect everywhere, we will stop creating a Rect directly and instead use
the new constructor methods. We'll also add getters for x and y because we access those
frequently, as shown here:

```rust
// src/engine.rs

impl Rect {
    ...
    pub fn x(&self) -> i16 {
        self.position.x
    }
    pub fn y(&self) -> i16 {
        self.position.y
    }

```
This gives you most of the tools you will need to fix the compiler errors. 
I won't reproduce all of them, because there are quite a few and it's repetitive. 

There are two examples you can use to make take care of all but one error. 
The first is replacing every reference to `.x` or `.y` with references to the methods.

This is how you do that in the intersects method of Rect :

```rust
// src/engine.rs

/*
    pub fn intersects(&self, rect: &Rect) -> bool {
        self.x < rect.right()
        && self.right() > rect.x
        && self.y < rect.bottom()
        && self.bottom() > rect.y
    }
*/
    pub fn intersects(&self, rect: &Rect) -> bool {
        self.x() < rect.right()
        && self.right() > rect.x()
        && self.y() < rect.bottom()
        && self.bottom() > rect.y()
    }
```

use the compiler and keep replacing 

```rust
// src/engine.rs

/*
    pub fn right(&self) -> i16 {
        self.x + self.width
    }
    pub fn bottom(&self) -> i16 {
        self.y + self.height
    }
*/
    pub fn right(&self) -> i16 {
        self.x() + self.width
    }
    pub fn bottom(&self) -> i16 {
        self.y() + self.height
    }
```

```rust
// src/engine.rs

impl Image {
    pub fn new(element: HtmlImageElement, position: Point) -> Self {
        let bounding_box = Rect {
            /*
            x: position.x.into(),
            y: position.y.into(),
            */
            position: Point{x: position.x.into(), y: position.y.into()},
            width: element.width() as i16, 
            height: element.height() as i16,
        };
        Self {
            element,
            position,
            bounding_box,
        }
    }
```

```rust
// src/engine.rs

impl Image {
    ...
    pub fn set_x(&mut self, x: i16) {
        //self.bounding_box.x = x;
        self.bounding_box.position.x = x;
        self.position.x = x;
    }
    ...
```
Humm Code smell!

We need to set the bounding_box.x value. Rather than using position.x, 
which will compile but expose us to errors if the internals of Rect change again, 
we'll add a setter to the Rect implementation, as shown here:


```rust
// src/engine.rs

impl Rect {
...
    pub fn set_x(&mut self, x: i16) {
        self.position.x = x
    }
}

```

Now, in Image , we can fix the last compiler error by using set_x , as shown here:


```rust
// src/engine.rs

impl Image {
    ...
    pub fn set_x(&mut self, x: i16) {
        //self.bounding_box.position.x = x;
        self.bounding_box.set_x(x);
        //self.position.x = x;
    }
...
```
Note::
    
    You may have noticed that the code is inconsistent when it uses setters versus
    when it uses public variables directly. In general, my rule of thumb is that
    dumb structures such as Rect don't need setters and getters, especially if we
    keep them immutable. However, if the internal structure changes, which it did
    here, then it's time to add an abstraction to hide the internals. This change,
    from x and y to a position, demonstrated the necessity of the setter after all.
    



```rust
// src/engine.rs

impl Renderer {
    pub fn clear(&self, rect: &Rect) {
        self.context.clear_rect(
            rect.x().into(),     // rect.x.into(),
            rect.y().into(),     // rect.y.into(),
            rect.width.into(),
            rect.height.into(),
        );
    }

```

```rust
// src/engine.rs

    pub fn draw_image(&self, 
                        image: &HtmlImageElement, 
                        frame: &Rect, 
                        destination: &Rect) {
        self.context
         .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &image,
            frame.x().into(),       // frame.x.into(),
            frame.y().into(),         // frame.y.into(),
            frame.width.into(),
            frame.height.into(),
            destination.x().into(),   //destination.x.into(),
            destination.y().into(),   //destination.y.into(),
            destination.width.into(),
            destination.height.into(),
        )
        .expect("Drawing is throwing exceptions! Unrecoverable error.");
    }//^-- draw_image
  
```

```rust
// src/engine.rs

...
    //for debuging
    pub fn draw_rect(&self, bounding_box: &Rect) {
        self.context.set_stroke_style(&JsValue::from_str("#FF0000"));
        self.context.begin_path();
        self.context.rect(
            bounding_box.x().into(),      // bounding_box.x.into(),
            bounding_box.y().into(),      // bounding_box.y.into(),
            bounding_box.width.into(),
            bounding_box.height.into(),
        );
        self.context.stroke();
    }
}//^-- impl Renderer

```


```rust
// src/game.rs

impl Platform {
    ...
    fn destination_box(&self) ->Rect {
        let platform = self
                        .sheet
                        .frames
                        .get("13.png")
                        .expect("13.png does not exist");
        
        Rect {
            // x: self.position.x.into(),
            // y: self.position.y.into(),
            position: Point{ 
                x: self.position.x.into(), 
                y: self.position.y.into()
            },                       
            width: (platform.frame.w * 3).into(),
            height: platform.frame.h.into(),
        }
    }//^-- destination_box

    fn bounding_boxes(&self) -> Vec<Rect> {
        ...
            let bounding_box_one = Rect {
            // x: destination_box.x,
            //y: destination_box.y,
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
            width: destination_box.width - (X_OFFSET * 2), //2.0),
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

    }//^-- fn bounding_boxes
    ...
    fn draw(&self, renderer: &Renderer) {
        renderer.draw_image( &self.image,
                             &Rect {
                                 position: Point {
                                     x: platform.frame.x.into(),
                                     y: platform.frame.y.into(),
                                 },
                                 width: (platform.frame.w * 3).into(),
                                 height: platform.frame.h.into(),
                             },
                             &self.destination_box(), //&self.bounding_box(),
                           );
    }//^-- draw
```



```rust
// src/game.rs

impl RedHatBoy {
    ...
    fn bounding_box(&self) -> Rect {
        const X_OFFSET: i16 = 18;     // f32 = 18.0;
        const Y_OFFSET: i16 = 14;     // f32 = 14.0;
        const WIDTH_OFFSET: i16 = 28; // f32 = 28.0;
        let mut bounding_box = self.destination_box();
        /*        
        bounding_box.x += X_OFFSET;
        bounding_box.width -= WIDTH_OFFSET;
        bounding_box.y += Y_OFFSET;
        bounding_box.height -= Y_OFFSET;
        */
        bounding_box.position.x += X_OFFSET;
        bounding_box.width -= WIDTH_OFFSET;
        bounding_box.position.y += Y_OFFSET;
        bounding_box.height -= Y_OFFSET;
        bounding_box
    }
...
```



```rust
// src/engine.rs

impl RedHatBoy {
    ...

    fn destination_box(&self) -> Rect {
        let sprite = self.current_sprite().expect("Cell not found");

        Rect {
            /*
            x: (self.state_machine.context().position.x + sprite.sprite_source_size.x as i16)
                .into(),
            y: (self.state_machine.context().position.y + sprite.sprite_source_size.y as i16)
                .into(),
            */
            position: Point {
                x: (self.state_machine.context().position.x + sprite.sprite_source_size.x) // as i16)
                .into(),
                y: (self.state_machine.context().position.y + sprite.sprite_source_size.y) //  as i16)
                .into(),
            },
            width: sprite.frame.w.into(),
            height: sprite.frame.h.into(),
        }
    }
```


```rust
// src/engine.rs

impl RedHatBoy {
    ...
    fn draw(&self, renderer: &Renderer) {
        let sprite = self.current_sprite().expect("Cell not found");
        
        renderer.draw_image(
            &self.image,
            &Rect {
                //x: sprite.frame.x.into(),
                //y: sprite.frame.y.into(),
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

```



At this point, you should see RHB running to the right and jumping on and off the
platform again. Make sure you check out this behavior each time you get to a successful
compile since it is easy to make a mistake as you make a large number of small changes.

----

Now that we've prepared `Rect` to hold a `position`, we can remove the **duplication**
of that data in Image. 

We'll start by removing `position` from the `Image` struct, as shown here:

```rust
// src/engine.rs

pub struct Image {
    element: HtmlImageElement,
    //position: Point,
    bounding_box: Rect,
}

```

Now, let's follow the compiler and remove all references to position in the Image
implementation. Fortunately, there are no longer any references to position outside
of the Image implementation, so we can do this by making a few quick changes. 
These changes are shown here. 
Note how wherever we previously used position, 
we are now using `bounding_box.position` or `bounding_box.x()` :


```rust
// src/engine.rs

impl Image {
    pub fn new(element: HtmlImageElement, position: Point) -> Self {
        let bounding_box = Rect {
            /*
            x: position.x.into(),
            y: position.y.into(),
            */
            position: Point{x: position.x.into(), y: position.y.into()},
            width: element.width() as i16, 
            height: element.height() as i16,
        };
        Self {
            element,
            //position,
            bounding_box,
        }
    }


```

```rust
// src/engine.rs


impl Image {
    ...
    pub fn draw(&self, renderer: &Renderer) {
        renderer.draw_entire_image(&self.element, &self.bounding_box.position); //position);
    }
    ...
```

`set_x` was a bit noisy. 

```rust
// src/engine.rs

impl Image {
    ...
    pub fn set_x(&mut self, x: i16) {
        self.bounding_box.set_x(x);
    }
...
```
-------------

```rust
// src/engine.rs


```


