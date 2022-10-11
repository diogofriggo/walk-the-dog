#[macro_use]
mod browser;
mod engine;
mod game;
mod segments;

use std::rc::Rc;

use engine::GameLoop;
use engine::Image;
use engine::KeyState;
use engine::Point;
use engine::SpriteSheet;
use game::Cell;
use game::RedHatBoy;
use game::Walk;
use game::WalkTheDog;
use wasm_bindgen::prelude::*;

use crate::engine::{Game, Renderer};
use crate::game::Sheet;
use anyhow::{anyhow, Result};
use async_trait::async_trait;

const LOW_PLATFORM: i16 = 420;
const HIGH_PLATFORM: i16 = 375;
const FIRST_PLATFORM: i16 = 500;
const TIMELINE_MINIMUM: i16 = 1000;

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
                // let stone = Image::new(stone, Point { x: 250, y: 546 });
                // let stone = Barrier::new(stone);

                let tiles = browser::fetch_json("tiles.json").await?;
                let tiles = tiles.into_serde::<Sheet>()?;

                let sheet = SpriteSheet {
                    sheet: tiles,
                    image: engine::load_image("tiles.png").await?,
                };
                let sheet = Rc::new(sheet);

                // let position = Point {
                //     x: FIRST_PLATFORM,
                //     y: LOW_PLATFORM,
                // };

                // let sprite_names = &["13.png", "14.png", "15.png"];
                // let first = Rect::new_from_x_y(0, 0, 60, 54);
                // let second = Rect::new_from_x_y(60, 0, 384 - (60 * 2), 93);
                // let third = Rect::new_from_x_y(384 - 60, 0, 60, 54);
                // let bounding_boxes = &[first, second, third];

                // let platform = Platform::new(sheet.clone(), position, sprite_names, bounding_boxes);
                let starting_obstacles =
                    segments::stone_and_platform(stone.clone(), sheet.clone(), 0);
                let timeline = game::rightmost(&starting_obstacles);

                let walk = Walk {
                    boy,
                    backgrounds: [first_background, second_background],
                    obstacles: starting_obstacles,
                    obstacle_sheet: sheet,
                    stone,
                    timeline,
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

            if walk.timeline < TIMELINE_MINIMUM {
                walk.generate_next_segment();
            } else {
                walk.timeline += velocity;
            }
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

    fn right(&self) -> i16 {
        self.image.bounding_box().right()
    }
}

pub struct Platform {
    sheet: Rc<SpriteSheet>,
    bounding_boxes: Vec<Rect>,
    sprites: Vec<Cell>,
    position: Point,
}

impl Platform {
    pub fn new(
        sheet: Rc<SpriteSheet>,
        position: Point,
        sprite_names: &[&str],
        bounding_boxes: &[Rect],
    ) -> Self {
        let sprites = sprite_names
            .iter()
            // Cloned turns Option<&T> into Option<T>
            .filter_map(|sprite_name| sheet.cell(sprite_name).cloned())
            .collect();

        // We are making bounding boxes be referenced by their image
        // This will screw up my draw_rect
        let bounding_boxes = bounding_boxes
            .iter()
            .map(|bounding_box| {
                let x = bounding_box.x() + position.x;
                let y = bounding_box.y() + position.y;
                Rect::new_from_x_y(x, y, bounding_box.width, bounding_box.height)
            })
            .collect();

        Platform {
            sheet,
            bounding_boxes,
            sprites,
            position,
        }
    }

    // pub fn draw_bounding_boxes(&self, renderer: &Renderer) {
    //     for bounding_box in &self.bounding_boxes {
    //         // TODO: this won't work anymore
    //         renderer.draw_rect(bounding_box);
    //     }
    // }

    pub fn bounding_boxes(&self) -> &Vec<Rect> {
        &self.bounding_boxes
        // const X_OFFSET: i16 = 60;
        // const END_HEIGHT: i16 = 54;
        // let destination_box = self.destination_box();
        // let position = Point {
        //     x: destination_box.x(),
        //     y: destination_box.y(),
        // };
        // let bounding_box_one = Rect::new(position, X_OFFSET, END_HEIGHT);

        // let position = Point {
        //     x: destination_box.x() + X_OFFSET,
        //     y: destination_box.y(),
        // };
        // let width = destination_box.width - (X_OFFSET * 2);
        // let bounding_box_two = Rect::new(position, width, destination_box.height);

        // let position = Point {
        //     x: destination_box.x() + destination_box.width - X_OFFSET,
        //     y: destination_box.y(),
        // };
        // let bounding_box_three = Rect::new(position, X_OFFSET, END_HEIGHT);

        // vec![bounding_box_one, bounding_box_two, bounding_box_three]
    }

    // could delete but this is still used by check_intersection
    pub fn destination_box(&self) -> Rect {
        let platform = self.current_sprite().expect("13.png does not exist");

        let position = Point {
            x: self.position.x,
            y: self.position.y,
        };
        let width = (platform.frame.w * 3) as i16;
        let height = platform.frame.h as i16;
        Rect::new(position, width, height)
    }

    pub fn current_sprite(&self) -> Option<&Cell> {
        self.sheet.cell("13.png")
    }
}

pub trait Obstacle {
    fn check_intersection(&self, boy: &mut RedHatBoy);
    fn draw(&self, renderer: &Renderer);
    fn move_horizontally(&mut self, x: i16);
    fn right(&self) -> i16;
}

impl Obstacle for Platform {
    fn check_intersection(&self, boy: &mut RedHatBoy) {
        if let Some(box_to_land_on) = self
            .bounding_boxes()
            .iter()
            .find(|&bounding_box| boy.bounding_box().intersects(bounding_box))
        {
            // remember positive velocity means going down
            // and if y1 < y2 it means that y1 is above y2
            let is_falling = boy.velocity_y() > 0;
            let is_above_platform = boy.pos_y() < self.destination_box().y();

            if is_falling && is_above_platform {
                let position = box_to_land_on.y();
                boy.land_on(position);
            } else {
                boy.knock_out();
            }
        }
    }

    fn draw(&self, renderer: &Renderer) {
        let mut x = 0;
        self.sprites.iter().for_each(|sprite| {
            let rect_x = sprite.frame.x as i16;
            let rect_y = sprite.frame.y as i16;
            let width = sprite.frame.w as i16;
            let height = sprite.frame.h as i16;
            let source = Rect::new_from_x_y(rect_x, rect_y, width, height);

            let rect_x = self.position.x + x;
            let rect_y = self.position.y;
            let width = sprite.frame.w as i16;
            let height = sprite.frame.h as i16;
            let destination = Rect::new_from_x_y(rect_x, rect_y, width, height);

            self.sheet.draw(renderer, &source, &destination);

            x += sprite.frame.w as i16;
        });

        // let platform = self.current_sprite().expect("13.png does not exist");

        // let destination = self.destination_box();

        // let position = Point {
        //     x: platform.frame.x as i16,
        //     y: platform.frame.y as i16,
        // };
        // let source = Rect::new(position, destination.width, destination.height);

        // self.sheet.draw(renderer, &source, &destination);
        // self.draw_bounding_boxes(renderer);
    }

    fn move_horizontally(&mut self, x: i16) {
        self.position.x += x;
        self.bounding_boxes.iter_mut().for_each(|bounding_box| {
            bounding_box.set_x(bounding_box.position.x + x);
        });
    }

    fn right(&self) -> i16 {
        self.bounding_boxes.last().unwrap().right()
    }
}

pub struct Rect {
    pub position: Point,
    pub width: i16,
    pub height: i16,
}

impl Rect {
    pub fn new(position: Point, width: i16, height: i16) -> Self {
        Rect {
            position,
            width,
            height,
        }
    }

    pub fn new_from_x_y(x: i16, y: i16, width: i16, height: i16) -> Self {
        let position = Point { x, y };
        Self::new(position, width, height)
    }

    pub fn intersects(&self, rect: &Rect) -> bool {
        let x_overlaps = self.left() < rect.right() && self.right() > rect.left();
        let y_overlaps = self.top() < rect.bottom() && self.bottom() > rect.top();
        x_overlaps && y_overlaps
    }

    pub fn left(&self) -> i16 {
        self.x()
    }

    pub fn right(&self) -> i16 {
        self.x() + self.width
    }

    pub fn bottom(&self) -> i16 {
        self.y() + self.height
    }

    pub fn top(&self) -> i16 {
        self.y()
    }

    pub fn x(&self) -> i16 {
        self.position.x
    }

    pub fn y(&self) -> i16 {
        self.position.y
    }

    pub fn width(&self) -> i16 {
        self.width
    }

    pub fn height(&self) -> i16 {
        self.height
    }

    pub fn set_x(&mut self, x: i16) {
        self.position.x = x;
    }
}
