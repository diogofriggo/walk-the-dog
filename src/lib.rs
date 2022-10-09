#[macro_use]
mod browser;
mod engine;
mod game;

use engine::GameLoop;
use engine::Image;
use engine::KeyState;
use engine::Point;
use game::Rect;
use game::RedHatBoy;
use game::Walk;
use game::WalkTheDog;
use wasm_bindgen::prelude::*;

use crate::engine::{Game, Renderer};
use crate::game::Sheet;
use anyhow::{anyhow, Result};
use async_trait::async_trait;

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                let json = browser::fetch_json("rhb.json").await?;

                let background = engine::load_image("BG.png").await?;
                let background = Image::new(background, Point { x: 0, y: 0 });

                let stone = engine::load_image("Stone.png").await?;
                let stone = Image::new(stone, Point { x: 150, y: 546 });

                let boy = RedHatBoy::new(
                    json.into_serde::<Sheet>()?,
                    engine::load_image("rhb.png").await?,
                );

                let walk = Walk {
                    boy,
                    background,
                    stone,
                };

                Ok(Box::new(WalkTheDog::Loaded(walk)))
            }
            WalkTheDog::Loaded(_) => Err(anyhow!("Error: Game is already initialized!")),
        }
    }

    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(walk) = self {
            let mut velocity = Point { x: 0, y: 0 };
            if keystate.is_pressed("ArrowDown") {
                walk.boy.slide();
            }

            if keystate.is_pressed("ArrowUp") {
                velocity.y -= 3;
            }

            if keystate.is_pressed("ArrowRight") {
                velocity.x += 3;
                walk.boy.run_right();
            }

            if keystate.is_pressed("ArrowLeft") {
                velocity.x -= 3;
            }

            if keystate.is_pressed("Space") {
                walk.boy.jump()
            }

            walk.boy.update();

            if walk
                .boy
                .bounding_box()
                .intersects(walk.stone.bounding_box())
            {
                walk.boy.knock_out();
            }
        }
    }

    fn draw(&self, renderer: &Renderer) {
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 600.0,
            height: 600.0,
        };

        renderer.clear(&rect);

        if let WalkTheDog::Loaded(walk) = self {
            walk.background.draw(renderer);
            walk.boy.draw(renderer);
            walk.stone.draw(renderer);
            walk.stone.draw_bounding_box(renderer);
        }
    }
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    browser::spawn_local(async move {
        let game = WalkTheDog::new();
        GameLoop::start(game)
            .await
            .expect("Could not start a game loop");
    });

    Ok(())
}
