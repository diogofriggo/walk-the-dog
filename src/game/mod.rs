#[cfg(test)]
mod test_browser;
// #[cfg(test)]
// use test_browser as browser;

use crate::browser;

mod red_hat_boy_states;

use std::{collections::HashMap, rc::Rc};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::channel::mpsc::UnboundedReceiver;
use rand::prelude::*;
use serde::Deserialize;
use web_sys::HtmlImageElement;

use crate::{
    engine::{self, Audio, Game, Image, KeyState, Point, Rect, Renderer, Sound, SpriteSheet},
    segments::{self, stone_and_platform},
};

const WIDTH: i16 = 1200;
const HEIGHT: i16 = 600;
const X_OFFSET: i16 = 18;
const Y_OFFSET: i16 = 14;
const WIDTH_OFFSET: i16 = 28;
const OBSTACLE_BUFFER: i16 = 20;
const TIMELINE_MINIMUM: i16 = 1000;

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

pub struct WalkTheDog {
    pub machine: Option<WalkTheDogStateMachine>,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog { machine: None }
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self.machine {
            None => {
                let json = browser::fetch_json("rhb.json").await?;

                let audio = Audio::new()?;
                let jump_sound = audio.load_sound("SFX_Jump_23.mp3").await?;
                let background_sound = audio.load_sound("background_song.mp3").await?;
                audio.play_looping_sound(&background_sound)?;

                let boy = RedHatBoy::new(
                    json.into_serde::<Sheet>()?,
                    engine::load_image("rhb.png").await?,
                    audio,
                    jump_sound,
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

                let tiles = browser::fetch_json("tiles.json").await?;
                let tiles = tiles.into_serde::<Sheet>()?;

                let sheet = SpriteSheet {
                    sheet: tiles,
                    image: engine::load_image("tiles.png").await?,
                };
                let sheet = Rc::new(sheet);

                let starting_obstacles =
                    segments::stone_and_platform(stone.clone(), sheet.clone(), 0);
                let timeline = rightmost(&starting_obstacles);

                let walk = Walk {
                    boy,
                    backgrounds: [first_background, second_background],
                    obstacles: starting_obstacles,
                    obstacle_sheet: sheet,
                    stone,
                    timeline,
                };

                Ok(Box::new(WalkTheDog {
                    machine: Some(WalkTheDogStateMachine::new(walk)),
                }))
            }
            Some(_) => Err(anyhow!("Error: Game is already initialized!")),
        }
    }

    fn update(&mut self, keystate: &KeyState) {
        if let Some(machine) = self.machine.take() {
            self.machine.replace(machine.update(keystate));
        }
        assert!(self.machine.is_some());
    }

    fn draw(&self, renderer: &Renderer) {
        let rect = Rect::new_from_x_y(0, 0, WIDTH, HEIGHT);
        renderer.clear(&rect);

        if let Some(machine) = &self.machine {
            machine.draw(renderer);
        }
    }
}

pub enum WalkTheDogStateMachine {
    Ready(WalkTheDogState<Ready>),
    Walking(WalkTheDogState<Walking>),
    GameOver(WalkTheDogState<GameOver>),
}

impl WalkTheDogStateMachine {
    fn new(walk: Walk) -> Self {
        WalkTheDogStateMachine::Ready(WalkTheDogState::new(walk))
    }

    fn update(self, keystate: &KeyState) -> Self {
        // log!("KeyState is {:#?}", keystate);
        match self {
            WalkTheDogStateMachine::Ready(state) => state.update(keystate).into(),
            WalkTheDogStateMachine::Walking(state) => state.update(keystate).into(),
            WalkTheDogStateMachine::GameOver(state) => state.update().into(),
        }
    }

    fn draw(&self, renderer: &Renderer) {
        match self {
            WalkTheDogStateMachine::Ready(state) => state.draw(renderer),
            WalkTheDogStateMachine::Walking(state) => state.draw(renderer),
            WalkTheDogStateMachine::GameOver(state) => state.draw(renderer),
        }
    }
}

pub struct WalkTheDogState<T> {
    _state: T,
    walk: Walk,
}

impl<T> WalkTheDogState<T> {
    fn draw(&self, renderer: &Renderer) {
        self.walk.draw(renderer);
    }
}

pub struct Ready;

pub struct Walking;

pub struct GameOver {
    new_game_event: UnboundedReceiver<()>,
}

impl GameOver {
    fn new_game_pressed(&mut self) -> bool {
        matches!(self.new_game_event.try_next(), Ok(Some(())))
    }
}

impl WalkTheDogState<Ready> {
    fn new(walk: Walk) -> WalkTheDogState<Ready> {
        WalkTheDogState {
            _state: Ready,
            walk,
        }
    }

    fn update(mut self, keystate: &KeyState) -> ReadyEndState {
        self.walk.boy.update();
        if keystate.is_pressed("ArrowRight") {
            ReadyEndState::Complete(self.start_running())
        } else {
            ReadyEndState::Continue(self)
        }
    }

    fn start_running(mut self) -> WalkTheDogState<Walking> {
        self.run_right();
        WalkTheDogState {
            _state: Walking,
            walk: self.walk,
        }
    }

    fn run_right(&mut self) {
        self.walk.boy.run_right();
    }
}

enum ReadyEndState {
    Complete(WalkTheDogState<Walking>),
    Continue(WalkTheDogState<Ready>),
}

impl From<ReadyEndState> for WalkTheDogStateMachine {
    fn from(state: ReadyEndState) -> Self {
        match state {
            ReadyEndState::Complete(walking) => walking.into(),
            ReadyEndState::Continue(ready) => ready.into(),
        }
    }
}

impl WalkTheDogState<Walking> {
    fn update(mut self, keystate: &KeyState) -> WalkingEndState {
        if keystate.is_pressed("Space") {
            self.walk.boy.jump()
        }

        if keystate.is_pressed("ArrowDown") {
            self.walk.boy.slide()
        }

        self.walk.boy.update();

        let walking_speed = self.walk.velocity();
        let [first_background, second_background] = &mut self.walk.backgrounds;

        first_background.move_horizontally(walking_speed);
        second_background.move_horizontally(walking_speed);

        if first_background.right() < 0 {
            first_background.set_x(second_background.right())
        }

        if second_background.right() < 0 {
            second_background.set_x(first_background.right())
        }

        self.walk.obstacles.iter_mut().for_each(|obstacle| {
            obstacle.move_horizontally(walking_speed);
            obstacle.check_intersection(&mut self.walk.boy);
        });

        if self.walk.timeline < TIMELINE_MINIMUM {
            self.walk.generate_next_segment();
        } else {
            self.walk.timeline += walking_speed;
        }

        if self.walk.is_dead() {
            WalkingEndState::Complete(self.end_game())
        } else {
            WalkingEndState::Continue(self)
        }
    }

    fn end_game(self) -> WalkTheDogState<GameOver> {
        let receiver = browser::draw_ui("<button id='new_game'>New Game</button>")
            .and_then(|_unit| browser::find_html_element_by_id("new_game"))
            .map(engine::add_click_handler)
            .unwrap();

        WalkTheDogState {
            _state: GameOver {
                new_game_event: receiver,
            },
            walk: self.walk,
        }
    }
}

enum WalkingEndState {
    Complete(WalkTheDogState<GameOver>),
    Continue(WalkTheDogState<Walking>),
}

impl WalkTheDogState<GameOver> {
    fn update(mut self) -> GameOverEndState {
        if self._state.new_game_pressed() {
            GameOverEndState::Complete(self.new_game())
        } else {
            GameOverEndState::Continue(self)
        }
    }

    fn new_game(self) -> WalkTheDogState<Ready> {
        if let Err(err) = browser::hide_ui() {
            error!("Error hiding the browser {:#?}", err);
        }

        WalkTheDogState {
            _state: Ready,
            walk: Walk::reset(self.walk),
        }
    }
}

enum GameOverEndState {
    Complete(WalkTheDogState<Ready>),
    Continue(WalkTheDogState<GameOver>),
}

impl From<WalkTheDogState<Ready>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<Ready>) -> Self {
        WalkTheDogStateMachine::Ready(state)
    }
}

impl From<WalkTheDogState<Walking>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<Walking>) -> Self {
        WalkTheDogStateMachine::Walking(state)
    }
}

impl From<WalkTheDogState<GameOver>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<GameOver>) -> Self {
        WalkTheDogStateMachine::GameOver(state)
    }
}

impl From<WalkingEndState> for WalkTheDogStateMachine {
    fn from(end_state: WalkingEndState) -> Self {
        match end_state {
            WalkingEndState::Continue(state) => WalkTheDogStateMachine::Walking(state),
            WalkingEndState::Complete(state) => WalkTheDogStateMachine::GameOver(state),
        }
    }
}

impl From<GameOverEndState> for WalkTheDogStateMachine {
    fn from(end_state: GameOverEndState) -> Self {
        match end_state {
            GameOverEndState::Continue(state) => WalkTheDogStateMachine::GameOver(state),
            GameOverEndState::Complete(state) => WalkTheDogStateMachine::Ready(state),
        }
    }
}

pub struct Walk {
    pub boy: RedHatBoy,
    pub backgrounds: [Image; 2],
    pub obstacles: Vec<Box<dyn Obstacle>>,
    pub obstacle_sheet: Rc<SpriteSheet>,
    pub stone: HtmlImageElement,
    pub timeline: i16,
}

impl Walk {
    pub fn velocity(&self) -> i16 {
        -self.boy.walking_speed()
    }

    pub fn generate_next_segment(&mut self) {
        let mut rng = thread_rng();
        let next_segment = rng.gen_range(0..2);

        let mut next_obstacles = match next_segment {
            0 => segments::stone_and_platform(
                self.stone.clone(),
                self.obstacle_sheet.clone(),
                self.timeline + OBSTACLE_BUFFER,
            ),
            1 => segments::platform_and_stone(
                self.stone.clone(),
                self.obstacle_sheet.clone(),
                self.timeline + OBSTACLE_BUFFER,
            ),
            _ => vec![],
        };

        self.timeline = rightmost(&next_obstacles);
        self.obstacles.append(&mut next_obstacles);
    }

    fn draw(&self, renderer: &Renderer) {
        self.backgrounds.iter().for_each(|background| {
            background.draw(renderer);
        });

        self.boy.draw(renderer);

        self.obstacles.iter().for_each(|obstacle| {
            obstacle.draw(renderer);
        });
    }

    fn is_dead(&self) -> bool {
        self.boy.knocked_out()
    }

    fn reset(walk: Self) -> Self {
        let starting_obstacles =
            stone_and_platform(walk.stone.clone(), walk.obstacle_sheet.clone(), 0);
        let timeline = rightmost(&starting_obstacles);

        Walk {
            boy: RedHatBoy::reset(walk.boy),
            backgrounds: walk.backgrounds,
            obstacles: starting_obstacles,
            obstacle_sheet: walk.obstacle_sheet,
            stone: walk.stone,
            timeline,
        }
    }
}

use red_hat_boy_states::*;

pub struct RedHatBoy {
    state_machine: RedHatBoyStateMachine,
    sprite_sheet: Sheet,
    image: HtmlImageElement,
}

impl RedHatBoy {
    pub fn new(
        sprite_sheet: Sheet,
        image: HtmlImageElement,
        audio: Audio,
        jump_sound: Sound,
    ) -> Self {
        RedHatBoy {
            state_machine: RedHatBoyStateMachine::Idle(RedHatBoyState::new(audio, jump_sound)),
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
        // renderer.draw_rect(&self.bounding_box());
    }

    pub fn bounding_box(&self) -> Rect {
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
        self.state_machine = self.state_machine.clone().transition(Event::Update)
    }

    pub fn run_right(&mut self) {
        self.state_machine = self.state_machine.clone().transition(Event::Run);
    }

    pub fn slide(&mut self) {
        self.state_machine = self.state_machine.clone().transition(Event::Slide);
    }

    pub fn jump(&mut self) {
        self.state_machine = self.state_machine.clone().transition(Event::Jump);
    }

    pub fn knock_out(&mut self) {
        // error!("Knock out!");
        // panic!();
        self.state_machine = self.state_machine.clone().transition(Event::KnockOut);
    }

    pub fn knocked_out(&self) -> bool {
        self.state_machine.is_dead()
    }

    pub fn land_on(&mut self, position: i16) {
        self.state_machine = self.state_machine.clone().transition(Event::Land(position));
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

    pub fn reset(boy: Self) -> Self {
        let audio = boy.state_machine.context().audio.clone();
        let jump_sound = boy.state_machine.context().jump_sound.clone();
        RedHatBoy::new(boy.sprite_sheet, boy.image, audio, jump_sound)
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

#[derive(Clone)]
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
        match (self.clone(), event) {
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

    fn is_dead(&self) -> bool {
        matches!(self, RedHatBoyStateMachine::Dead(_))
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

pub fn rightmost(obstacle_list: &[Box<dyn Obstacle>]) -> i16 {
    obstacle_list
        .iter()
        .map(|obstacle| obstacle.right())
        .max_by(|x, y| x.cmp(y))
        .unwrap_or(0)
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
