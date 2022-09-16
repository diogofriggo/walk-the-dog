#[macro_use]
mod browser;
mod engine;
mod game;

use engine::GameLoop;
use engine::KeyState;
use engine::Point;
use engine::WalkTheDog;
use game::Rect;
use wasm_bindgen::prelude::*;

use crate::engine::{Game, Renderer};
use crate::game::Sheet;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        let sheet: Sheet = browser::fetch_json("rhb.json").await?.into_serde()?;

        let image = engine::load_image("rhb.png").await?;

        Ok(Box::new(WalkTheDog {
            image: Some(image),
            sheet: Some(sheet),
            frame: self.frame,
            position: self.position,
        }))
    }

    fn update(&mut self, keystate: &KeyState) {
        let mut velocity = Point { x: 0, y: 0 };
        if keystate.is_pressed("ArrowDown") {
            velocity.y += 3;
        }

        if keystate.is_pressed("ArrowUp") {
            velocity.y -= 3;
        }

        if keystate.is_pressed("ArrowRight") {
            velocity.x += 3;
        }

        if keystate.is_pressed("ArrowLeft") {
            velocity.x -= 3;
        }

        if self.frame < 23 {
            self.frame += 1;
        } else {
            self.frame = 0;
        }

        self.position.x += velocity.x;
        self.position.y += velocity.y;
    }

    fn draw(&self, renderer: &Renderer) {
        let current_sprite = (self.frame / 3) + 1;
        let frame_name = format!("Run ({}).png", current_sprite);

        let sprite = self
            .sheet
            .as_ref()
            .and_then(|sheet| sheet.frames.get(&frame_name))
            .expect("Cell not found");

        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 600.0,
            height: 600.0,
        };

        renderer.clear(&rect);

        let frame = Rect {
            x: sprite.frame.x.into(),
            y: sprite.frame.y.into(),
            width: sprite.frame.w.into(),
            height: sprite.frame.h.into(),
        };

        let destination = Rect {
            x: self.position.x.into(),
            y: self.position.y.into(),
            width: sprite.frame.w.into(),
            height: sprite.frame.h.into(),
        };

        let _ = self
            .image
            .as_ref()
            .map(|image| renderer.draw_image(image, &frame, &destination));
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
