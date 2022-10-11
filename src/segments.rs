use std::rc::Rc;

use web_sys::HtmlImageElement;

use crate::{
    engine::{Image, Point, SpriteSheet},
    Barrier, Obstacle, Platform, Rect, FIRST_PLATFORM, HIGH_PLATFORM, LOW_PLATFORM,
};

pub fn stone_and_platform(
    stone: HtmlImageElement,
    sprite_sheet: Rc<SpriteSheet>,
    offset_x: i16,
) -> Vec<Box<dyn Obstacle>> {
    const INITIAL_STONE_OFFSET: i16 = 150;
    const STONE_ON_GROUND: i16 = 546;

    let position = Point {
        x: offset_x + INITIAL_STONE_OFFSET,
        y: STONE_ON_GROUND,
    };
    let stone = Image::new(stone, position);
    let stone = Barrier::new(stone);

    let position = Point {
        x: offset_x + FIRST_PLATFORM,
        y: LOW_PLATFORM,
    };
    vec![
        Box::new(stone),
        Box::new(create_floating_platform(sprite_sheet, position)),
    ]
}

pub fn platform_and_stone(
    stone: HtmlImageElement,
    sprite_sheet: Rc<SpriteSheet>,
    offset_x: i16,
) -> Vec<Box<dyn Obstacle>> {
    const INITIAL_STONE_OFFSET: i16 = 150;
    const STONE_ON_GROUND: i16 = 546;

    let position = Point {
        x: offset_x + INITIAL_STONE_OFFSET,
        y: STONE_ON_GROUND,
    };
    let stone = Image::new(stone, position);
    let stone = Barrier::new(stone);

    let position = Point {
        x: offset_x + FIRST_PLATFORM,
        y: HIGH_PLATFORM,
    };
    vec![
        Box::new(stone),
        Box::new(create_floating_platform(sprite_sheet, position)),
    ]
}

fn create_floating_platform(sprite_sheet: Rc<SpriteSheet>, position: Point) -> Platform {
    const FLOATING_PLATFORM_SPRITES: [&str; 3] = ["13.png", "14.png", "15.png"];
    let first = Rect::new_from_x_y(0, 0, 60, 54);
    let second = Rect::new_from_x_y(60, 0, 384 - (60 * 2), 93);
    let third = Rect::new_from_x_y(384 - 60, 0, 60, 54);
    let floating_platform_bounding_boxes: [Rect; 3] = [first, second, third];

    Platform::new(
        sprite_sheet,
        position,
        &FLOATING_PLATFORM_SPRITES,
        &floating_platform_bounding_boxes,
    )
}
