pub mod audiostream {
    tonic::include_proto!("audiostream");
}

use audiostream::{audio_streamer_client::AudioStreamerClient, EmptyRequest};
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
) -> Result<TrackStreamed, Box<dyn std::error::Error>> {
    let mut stream = client.stream_track(EmptyRequest {}).await?.into_inner();
    let mut data: Vec<f32> = Vec::new();
    let mut frequency: u32 = 0;
    let mut channels: u32 = 0;
    println!("STARTED STREAMING");
    while let Some(msg) = stream.message().await? {
        println!("MSG: {:?}",msg);
        data.push(msg.track_byte);
        frequency = msg.frequency;
        channels = msg.channels;
    }
    println!("DONE STREAMING");

    return Ok(TrackStreamed {
        frequency,
        channels,
        bytes: data,
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = AudioStreamerClient::connect("http://[::1]:50051").await?;

    // let downloaded_track: Track = get_track(&mut client).await?;
    let downloaded_track: TrackStreamed = get_track_streamed(&mut client).await?;

    let (_stream, stream_handler) =
        OutputStream::try_default().expect("Couln't obtain default playback device");
    let sink: Sink = Sink::try_new(&stream_handler).unwrap();
    let source = rodio::buffer::SamplesBuffer::new(
        downloaded_track.channels as u16,
        downloaded_track.frequency,
        downloaded_track.bytes,
    );
    sink.append(source);
    sink.set_volume(0.3);

    sink.sleep_until_end();

    return Ok(());
}
