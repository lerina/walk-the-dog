## Book out of sync with github

ex: 

in the book

```rust
// src/game.rs

impl RedHatBoyState<Jumping> {
    ...
    pub fn update(mut self) -> JumpingEndState {
        self.context = self.context.update(JUMPING_FRAMES);
        if self.context.position.y >= FLOOR {
            JumpingEndState::Complete(self.land())
        } else {
            JumpingEndState::Jumping(self)
        }
    }
}
```

in the repo for chapter 04

```rust
// src/game.rs

        pub fn update(mut self) -> JumpingEndState {
            self.update_context(JUMPING_FRAMES);

            if self.context.position.y >= FLOOR {
                JumpingEndState::Landing(self.land())
            } else {
                JumpingEndState::Jumping(self)
            }
        }
```


In order to make it ease to follow along with the repo when getting lost with
the sometimes confusing book, we are go int to sinc with the git repo. 
We made it this far and everything works. So its just naming changes really.


