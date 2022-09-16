use std::collections::HashMap;

use serde::Deserialize;

use crate::engine::{Point, WalkTheDog};

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Deserialize)]
pub struct SheetRect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

#[derive(Debug, Deserialize)]
pub struct Cell {
    pub frame: SheetRect,
}

#[derive(Debug, Deserialize)]
pub struct Sheet {
    pub frames: HashMap<String, Cell>,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog {
            image: None,
            sheet: None,
            frame: 0,
            position: Point { x: 0, y: 0 },
        }
    }
}
