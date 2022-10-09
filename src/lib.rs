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
use crate::game::{Platform, Sheet};
use anyhow::{anyhow, Result};
use async_trait::async_trait;

const LOW_PLATFORM: i16 = 420;
const HIGH_PLATFORM: i16 = 375;
const FIRST_PLATFORM: i16 = 370;

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                let json = browser::fetch_json("rhb.json").await?;

                let boy = RedHatBoy::new(
                    json.into_serde::<Sheet>()?,
                    engine::load_image("rhb.png").await?,
                );

                let background = engine::load_image("BG.png").await?;
                let background = Image::new(background, Point { x: 0, y: 0 });

                let stone = engine::load_image("Stone.png").await?;
                let stone = Image::new(stone, Point { x: 150, y: 546 });

                let platform_sheet = browser::fetch_json("tiles.json").await?;
                let platform_sheet = platform_sheet.into_serde::<Sheet>()?;

                let platform = engine::load_image("tiles.png").await?;
                let platform = Platform::new(
                    platform_sheet,
                    platform,
                    Point {
                        x: FIRST_PLATFORM,
                        y: LOW_PLATFORM,
                    },
                );

                let walk = Walk {
                    boy,
                    background,
                    stone,
                    platform,
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

            for bounding_box in &walk.platform.bounding_boxes() {
                let intersects_with_platform = walk.boy.bounding_box().intersects(bounding_box);

                if intersects_with_platform {
                    // remember positive velocity means going down
                    // and if y1 < y2 it means that y1 is above y2
                    let is_falling = walk.boy.velocity_y() > 0;
                    let is_above_platform =
                        walk.boy.pos_y() < (walk.platform.destination_box().y as i16);

                    if is_falling && is_above_platform {
                        let position = bounding_box.y;
                        walk.boy.land_on(position as i16);
                    } else {
                        walk.boy.knock_out();
                    }
                }
            }

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
            width: game::WIDTH as f32,
            height: game::HEIGHT as f32,
        };

        renderer.clear(&rect);

        if let WalkTheDog::Loaded(walk) = self {
            walk.background.draw(renderer);
            walk.boy.draw(renderer);
            walk.stone.draw(renderer);
            walk.stone.draw_bounding_box(renderer);
            walk.platform.draw(renderer);
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
