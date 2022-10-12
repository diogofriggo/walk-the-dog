// use anyhow::{anyhow, Result};
// use wasm_bindgen::JsValue;
// use web_sys::HtmlElement;

// pub fn draw_ui(html: &str) -> Result<()> {
//     Ok(())
// }

// pub fn hide_ui() -> Result<()> {
//     Ok(())
// }

// pub fn find_html_element_by_id(id: &str) -> Result<HtmlElement> {
//     Err(anyhow!("Not implemented yet"))
// }

// pub async fn fetch_json(json_path: &str) -> Result<JsValue> {
//     Err(anyhow!("Not implemented yet"))
// }

#[cfg(test)]
mod tests {
    use crate::{
        browser,
        engine::{Audio, Image, Point, Sound, SpriteSheet},
        game::{GameOver, RedHatBoy, Sheet, Walk, WalkTheDogState},
    };
    use futures::channel::mpsc::unbounded;
    use std::{collections::HashMap, rc::Rc};
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{AudioBuffer, AudioBufferOptions, HtmlImageElement};

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
    #[wasm_bindgen_test]
    fn test_transition_from_game_over_to_new_game() {
        let (_, receiver) = unbounded();
        let image = HtmlImageElement::new().unwrap();
        let audio = Audio::new().unwrap();
        let options = AudioBufferOptions::new(1, 3000.0);
        let sound = Sound {
            buffer: AudioBuffer::new(&options).unwrap(),
        };
        let rhb = RedHatBoy::new(
            Sheet {
                frames: HashMap::new(),
            },
            image.clone(),
            audio,
            sound,
        );

        let sprite_sheet = SpriteSheet {
            sheet: Sheet {
                frames: HashMap::new(),
            },
            image: image.clone(),
        };

        let walk = Walk {
            boy: rhb,
            backgrounds: [
                Image::new(image.clone(), Point { x: 0, y: 0 }),
                Image::new(image.clone(), Point { x: 0, y: 0 }),
            ],
            obstacles: vec![],
            obstacle_sheet: Rc::new(sprite_sheet),
            stone: image,
            timeline: 0,
        };

        // act

        let document = browser::document().unwrap();
        document
            .body()
            .unwrap()
            .insert_adjacent_html("afterbegin", "<canvas id='canvas'></canvas>")
            .unwrap();

        document
            .body()
            .unwrap()
            .insert_adjacent_html("afterbegin", "<div id='ui'></div>")
            .unwrap();
        browser::draw_ui("<p>This is the UI</p>").unwrap();

        let state = WalkTheDogState {
            _state: GameOver {
                new_game_event: receiver,
            },
            walk,
        };

        state.new_game();

        // assert
        let ui = browser::find_html_element_by_id("ui").unwrap();
        assert_eq!(ui.child_element_count(), 0);
    }
}
