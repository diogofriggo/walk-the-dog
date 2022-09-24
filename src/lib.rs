#[macro_use]
mod browser;
mod engine;
mod game;

use engine::GameLoop;
use engine::KeyState;
use engine::Point;
use game::Rect;
use game::RedHatBoy;
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

                let rhb = RedHatBoy::new(
                    json.into_serde::<Sheet>()?,
                    engine::load_image("rhb.png").await?,
                );

                Ok(Box::new(WalkTheDog::Loaded(rhb)))
            }
            WalkTheDog::Loaded(_) => Err(anyhow!("Error: Game is already initialized!")),
        }
    }

    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(rhb) = self {
            let mut velocity = Point { x: 0, y: 0 };
            if keystate.is_pressed("ArrowDown") {
                rhb.slide();
            }

            if keystate.is_pressed("ArrowUp") {
                velocity.y -= 3;
            }

            if keystate.is_pressed("ArrowRight") {
                velocity.x += 3;
                rhb.run_right();
            }

            if keystate.is_pressed("ArrowLeft") {
                velocity.x -= 3;
            }

            if keystate.is_pressed("Space") {
                rhb.jump()
            }

            rhb.update();
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

        if let WalkTheDog::Loaded(rhb) = self {
            rhb.draw(renderer);
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
