mod red_hat_boy_states;

use std::collections::HashMap;

use serde::Deserialize;
use web_sys::HtmlImageElement;

use crate::engine::{Image, Point, Renderer};

pub const WIDTH: i16 = 1200;
pub const HEIGHT: i16 = 600;

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

#[derive(Debug, Deserialize, Clone)]
pub struct SheetRect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    pub frame: SheetRect,
    pub sprite_source_size: SheetRect,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Sheet {
    pub frames: HashMap<String, Cell>,
}

pub enum WalkTheDog {
    Loading,
    Loaded(Walk),
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog::Loading
    }
}

pub struct Walk {
    pub boy: RedHatBoy,
    pub backgrounds: [Image; 2],
    pub obstacles: Vec<Box<dyn Obstacle>>,
}

impl Walk {
    pub fn velocity(&self) -> i16 {
        -self.boy.walking_speed()
    }
}

use red_hat_boy_states::*;

pub struct RedHatBoy {
    state_machine: RedHatBoyStateMachine,
    sprite_sheet: Sheet,
    image: HtmlImageElement,
}

impl RedHatBoy {
    pub fn new(sprite_sheet: Sheet, image: HtmlImageElement) -> Self {
        RedHatBoy {
            state_machine: RedHatBoyStateMachine::Idle(RedHatBoyState::new()),
            sprite_sheet,
            image,
        }
    }

    pub fn draw(&self, renderer: &Renderer) {
        let sprite = self.current_sprite().expect("Cell not found!");

        let position = Point {
            x: sprite.frame.x as i16,
            y: sprite.frame.y as i16,
        };
        let width = sprite.frame.w as i16;
        let height = sprite.frame.h as i16;
        let source = Rect::new(position, width, height);

        let destination = self.destination_box();

        renderer.draw_image(&self.image, &source, &destination);
        renderer.draw_rect(&self.bounding_box());
    }

    pub fn bounding_box(&self) -> Rect {
        const X_OFFSET: i16 = 18;
        const Y_OFFSET: i16 = 14;
        const WIDTH_OFFSET: i16 = 28;
        let mut bounding_box = self.destination_box();
        bounding_box.position.x += X_OFFSET;
        bounding_box.width -= WIDTH_OFFSET;
        bounding_box.position.y += Y_OFFSET;
        bounding_box.height -= Y_OFFSET;
        bounding_box
    }

    pub fn destination_box(&self) -> Rect {
        let sprite = self.current_sprite().expect("Cell not found!");

        let x = self.state_machine.context().position.x;
        let y = self.state_machine.context().position.y;
        let sprite_x = sprite.sprite_source_size.x as i16;
        let sprite_y = sprite.sprite_source_size.y as i16;

        let position = Point {
            x: x + sprite_x,
            y: y + sprite_y,
        };
        let width = sprite.frame.w as i16;
        let height = sprite.frame.h as i16;
        Rect::new(position, width, height)
    }

    pub fn current_sprite(&self) -> Option<&Cell> {
        self.sprite_sheet.frames.get(&self.frame_name())
    }

    pub fn frame_name(&self) -> String {
        let name = self.state_machine.frame_name();
        let number = (self.state_machine.context().frame / 3) + 1;
        format!("{} ({}).png", name, number)
    }

    pub fn update(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Update)
    }

    pub fn run_right(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Run);
    }

    pub fn slide(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Slide);
    }

    pub fn jump(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Jump);
    }

    pub fn knock_out(&mut self) {
        self.state_machine = self.state_machine.transition(Event::KnockOut);
    }

    pub fn land_on(&mut self, position: i16) {
        self.state_machine = self.state_machine.transition(Event::Land(position));
    }

    pub fn _velocity_x(&self) -> i16 {
        self.state_machine.context().velocity.x
    }

    pub fn velocity_y(&self) -> i16 {
        self.state_machine.context().velocity.y
    }

    pub fn _pos_x(&self) -> i16 {
        self.state_machine.context().position.x
    }

    pub fn pos_y(&self) -> i16 {
        self.state_machine.context().position.y
    }

    pub fn walking_speed(&self) -> i16 {
        self.state_machine.context().velocity.x
    }
}

pub enum Event {
    Run,
    Slide,
    Update,
    Jump,
    KnockOut,
    Land(i16),
}

#[derive(Copy, Clone)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    Running(RedHatBoyState<Running>),
    Sliding(RedHatBoyState<Sliding>),
    Jumping(RedHatBoyState<Jumping>),
    Falling(RedHatBoyState<Falling>),
    Dead(RedHatBoyState<Dead>),
}

impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            (RedHatBoyStateMachine::Idle(state), Event::Run) => state.run().into(),
            (RedHatBoyStateMachine::Idle(state), Event::Update) => state.update().into(),

            (RedHatBoyStateMachine::Running(state), Event::Slide) => state.slide().into(),
            (RedHatBoyStateMachine::Running(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Running(state), Event::Jump) => state.jump().into(),
            (RedHatBoyStateMachine::Running(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Running(state), Event::Land(position)) => {
                state.land_on(position).into()
            }

            (RedHatBoyStateMachine::Sliding(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Sliding(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Sliding(state), Event::Land(position)) => {
                state.land_on(position).into()
            }

            (RedHatBoyStateMachine::Jumping(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Jumping(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Jumping(state), Event::Land(position)) => {
                state.land_on(position).into()
            }

            (RedHatBoyStateMachine::Falling(state), Event::Update) => state.update().into(),

            _ => self,
        }
    }

    fn frame_name(&self) -> &str {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.frame_name(),
            RedHatBoyStateMachine::Running(state) => state.frame_name(),
            RedHatBoyStateMachine::Sliding(state) => state.frame_name(),
            RedHatBoyStateMachine::Jumping(state) => state.frame_name(),
            RedHatBoyStateMachine::Falling(state) => state.frame_name(),
            RedHatBoyStateMachine::Dead(state) => state.frame_name(),
        }
    }

    fn context(&self) -> &RedHatBoyContext {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.context(),
            RedHatBoyStateMachine::Running(state) => state.context(),
            RedHatBoyStateMachine::Sliding(state) => state.context(),
            RedHatBoyStateMachine::Jumping(state) => state.context(),
            RedHatBoyStateMachine::Falling(state) => state.context(),
            RedHatBoyStateMachine::Dead(state) => state.context(),
        }
    }
}

impl From<RedHatBoyState<Idle>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Idle>) -> Self {
        RedHatBoyStateMachine::Idle(state)
    }
}

impl From<RedHatBoyState<Running>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Running>) -> Self {
        RedHatBoyStateMachine::Running(state)
    }
}

impl From<RedHatBoyState<Sliding>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Sliding>) -> Self {
        RedHatBoyStateMachine::Sliding(state)
    }
}

impl From<RedHatBoyState<Jumping>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Jumping>) -> Self {
        RedHatBoyStateMachine::Jumping(state)
    }
}

impl From<RedHatBoyState<Falling>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Falling>) -> Self {
        RedHatBoyStateMachine::Falling(state)
    }
}

impl From<RedHatBoyState<Dead>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Dead>) -> Self {
        RedHatBoyStateMachine::Dead(state)
    }
}

impl From<SlidingEndState> for RedHatBoyStateMachine {
    fn from(end_state: SlidingEndState) -> Self {
        match end_state {
            SlidingEndState::Complete(running_state) => running_state.into(),
            SlidingEndState::Sliding(sliding_state) => sliding_state.into(),
        }
    }
}

impl From<JumpingEndState> for RedHatBoyStateMachine {
    fn from(end_state: JumpingEndState) -> Self {
        match end_state {
            JumpingEndState::Landing(running_state) => running_state.into(),
            JumpingEndState::Jumping(jumping_state) => jumping_state.into(),
        }
    }
}

impl From<FallingEndState> for RedHatBoyStateMachine {
    fn from(end_state: FallingEndState) -> Self {
        match end_state {
            FallingEndState::Complete(dead_state) => dead_state.into(),
            FallingEndState::Falling(falling_state) => falling_state.into(),
        }
    }
}

pub struct Platform {
    sheet: Sheet,
    image: HtmlImageElement,
    position: Point,
}

impl Platform {
    pub fn new(sheet: Sheet, image: HtmlImageElement, position: Point) -> Self {
        Platform {
            sheet,
            image,
            position,
        }
    }

    pub fn draw_bounding_boxes(&self, renderer: &Renderer) {
        for bounding_box in &self.bounding_boxes() {
            renderer.draw_rect(bounding_box);
        }
    }

    pub fn bounding_boxes(&self) -> Vec<Rect> {
        const X_OFFSET: i16 = 60;
        const END_HEIGHT: i16 = 54;
        let destination_box = self.destination_box();
        let position = Point {
            x: destination_box.x(),
            y: destination_box.y(),
        };
        let bounding_box_one = Rect::new(position, X_OFFSET, END_HEIGHT);

        let position = Point {
            x: destination_box.x() + X_OFFSET,
            y: destination_box.y(),
        };
        let width = destination_box.width - (X_OFFSET * 2);
        let bounding_box_two = Rect::new(position, width, destination_box.height);

        let position = Point {
            x: destination_box.x() + destination_box.width - X_OFFSET,
            y: destination_box.y(),
        };
        let bounding_box_three = Rect::new(position, X_OFFSET, END_HEIGHT);

        vec![bounding_box_one, bounding_box_two, bounding_box_three]
    }

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
        self.sheet.frames.get("13.png")
    }
}

pub trait Obstacle {
    fn check_intersection(&self, boy: &mut RedHatBoy);
    fn draw(&self, renderer: &Renderer);
    fn move_horizontally(&mut self, x: i16);
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
        let platform = self.current_sprite().expect("13.png does not exist");

        let destination = self.destination_box();

        let position = Point {
            x: platform.frame.x as i16,
            y: platform.frame.y as i16,
        };
        let source = Rect::new(position, destination.width, destination.height);

        renderer.draw_image(&self.image, &source, &destination);
        self.draw_bounding_boxes(renderer);
    }

    fn move_horizontally(&mut self, distance: i16) {
        for bounding_box in &mut self.bounding_boxes() {
            bounding_box.position.x += distance;
        }
        self.position.x += distance;
    }
}
