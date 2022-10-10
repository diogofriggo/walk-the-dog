#[macro_use]
mod browser;
mod engine;
mod game;

use engine::GameLoop;
use engine::Image;
use engine::KeyState;
use engine::Point;
use game::Obstacle;
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
const _HIGH_PLATFORM: i16 = 375;
const FIRST_PLATFORM: i16 = 500;

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
                let first_background = Image::new(background.clone(), Point { x: 0, y: 0 });
                let background_width = background.width() as i16;
                let second_background = Image::new(
                    background,
                    Point {
                        x: background_width,
                        y: 0,
                    },
                );

                let stone = engine::load_image("Stone.png").await?;
                let stone = Image::new(stone, Point { x: 250, y: 546 });
                let stone = Barrier::new(stone);

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
                    backgrounds: [first_background, second_background],
                    obstacles: vec![Box::new(stone), Box::new(platform)],
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

            let velocity = walk.velocity();
            let [first_background, second_background] = &mut walk.backgrounds;

            first_background.move_horizontally(velocity);
            second_background.move_horizontally(velocity);

            if first_background.right() < 0 {
                first_background.set_x(second_background.right())
            }

            if second_background.right() < 0 {
                second_background.set_x(first_background.right())
            }

            walk.obstacles.iter_mut().for_each(|obstacle| {
                obstacle.move_horizontally(velocity);
                obstacle.check_intersection(&mut walk.boy);
            });
        }
    }

    fn draw(&self, renderer: &Renderer) {
        let rect = Rect::new_from_x_y(0, 0, game::WIDTH, game::HEIGHT);
        renderer.clear(&rect);

        if let WalkTheDog::Loaded(walk) = self {
            for background in &walk.backgrounds {
                background.draw(renderer);
            }
            walk.boy.draw(renderer);
            walk.obstacles.iter().for_each(|obstacle| {
                obstacle.draw(renderer);
            });
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

pub struct Barrier {
    image: Image,
}

impl Barrier {
    pub fn new(image: Image) -> Self {
        Barrier { image }
    }
}

impl Obstacle for Barrier {
    fn check_intersection(&self, boy: &mut RedHatBoy) {
        if boy.bounding_box().intersects(self.image.bounding_box()) {
            boy.knock_out();
        }
    }

    fn draw(&self, renderer: &Renderer) {
        self.image.draw(renderer);
    }

    fn move_horizontally(&mut self, x: i16) {
        self.image.move_horizontally(x);
    }
}
