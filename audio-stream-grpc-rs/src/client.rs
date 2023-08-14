pub mod audiostream {
    tonic::include_proto!("audiostream");
}

use std::io::BufReader;

use audiostream::{audio_streamer_client::AudioStreamerClient, EmptyRequest, TrackRequest};
use rodio::{Decoder, OutputStream, Sink};
use tonic::transport::Channel;

struct TrackStreamed {
    frequency: u32,
    channels: u32,
    bytes: Vec<f32>,
}
//Error, message length too large:
// async fn get_track(
//     client: &mut AudioStreamerClient<Channel>,
// ) -> Result<Track, Box<dyn std::error::Error>> {
//     let response = client.get_track(EmptyRequest{}).await?.into_inner();
//
//     return Ok(response);
// }
async fn get_track_streamed(
    client: &mut AudioStreamerClient<Channel>,
    track: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handler) =
        OutputStream::try_default().expect("Couln't obtain default playback device");
    let sink: Sink = Sink::try_new(&stream_handler).unwrap();

    let mut stream = client
        .stream_track(TrackRequest {
            track_name: track.to_owned(),
        })
        .await?
        .into_inner();
    // let mut frequency: u32 = 0;
    // let mut channels: u32 = 0;
    println!("STARTED STREAMING");
    while let Some(msg) = stream.message().await? {
        let mut data: Vec<f32> = Vec::new();
        println!("msg");
        msg.track_byte
            .iter()
            .for_each(|byte| data.push(byte.clone()));
        // frequency = msg.frequency;
        // channels = msg.channels;
        let source = rodio::buffer::SamplesBuffer::new(msg.channels as u16, msg.frequency, data);
        sink.append(source);
        sink.set_volume(0.03);
        if sink.is_paused() {
            sink.play();
        }
    }
    while !sink.empty(){
        // println!("is paused? {}",sink.is_paused());
        // sink.play();
    }
    sink.stop();

    println!("DONE STREAMING");
    // println!("Length of streamed data={}", data.len());

    return Ok(());
}
async fn get_track_list(
    client: &mut AudioStreamerClient<Channel>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response = client.get_track_list(EmptyRequest {}).await?;
    return Ok(response.into_inner().track_names);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = AudioStreamerClient::connect("http://[::1]:50051").await?;
    let track_list: Vec<String> = get_track_list(&mut client).await?;
    println!("{:?}", track_list);

    // let downloaded_track: Track = get_track(&mut client).await?;
    _ = get_track_streamed(&mut client, track_list.get(1).unwrap()).await?;

    return Ok(());
}
