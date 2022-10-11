// const audioContext = new AudioContext();
// let sound = await fetch("SFX_Jump_23.mp3");
// let soundBuffer = await sound.arrayBuffer();
// let decodedArray = await audioContext.decodeAudioData(soundBuffer);

// let trackSource = audioContext.createBufferSource();
// trackSource.buffer = decodedArray;
// trackSource.connect(audioContext.destination);
// trackSource.start();

use anyhow::{anyhow, Result};
use js_sys::ArrayBuffer;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, AudioDestinationNode, AudioNode};

pub async fn decode_audio_data(
    ctx: &AudioContext,
    array_buffer: &ArrayBuffer,
) -> Result<AudioBuffer> {
    let promise = ctx
        .decode_audio_data(array_buffer)
        .map_err(|err| anyhow!("Could not decode audio from array buffer {:#?}", err))?;

    JsFuture::from(promise)
        .await
        .map_err(|err| anyhow!("Could not convert promise to future {:#?}", err))?
        .dyn_into()
        .map_err(|err| anyhow!("Could not cast into AudioBuffer {:#?}", err))
}

pub enum LOOPING {
    NO,
    YES,
}

pub fn play_sound(ctx: &AudioContext, buffer: &AudioBuffer, looping: LOOPING) -> Result<()> {
    let track_source = create_track_source(ctx, buffer)?;

    if matches!(looping, LOOPING::YES) {
        track_source.set_loop(true);
    }

    track_source.start()
        .map_err(|err| anyhow!("Could not start sound! {:#?}", err))
}

fn create_track_source(ctx: &AudioContext, buffer: &AudioBuffer) -> Result<AudioBufferSourceNode> {
    let track_source = create_buffer_source(ctx)?;
    track_source.set_buffer(Some(&buffer));
    connect_with_audio_node(&track_source, &ctx.destination())?;
    Ok(track_source)
}

pub fn create_audio_context() -> Result<AudioContext> {
    AudioContext::new().map_err(|err| anyhow!("Could not create audio context: {:#?}", err))
}

fn create_buffer_source(ctx: &AudioContext) -> Result<AudioBufferSourceNode> {
    ctx.create_buffer_source()
        .map_err(|err| anyhow!("Error creating buffer source {:#?}", err))
}

fn connect_with_audio_node(
    buffer_source: &AudioBufferSourceNode,
    destination: &AudioDestinationNode,
) -> Result<AudioNode> {
    buffer_source
        .connect_with_audio_node(destination)
        .map_err(|err| anyhow!("Error connecting audio source to destination {:#?}", err))
}
