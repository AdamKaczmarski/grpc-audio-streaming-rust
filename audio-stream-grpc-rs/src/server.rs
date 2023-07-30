pub mod audiostream {
    tonic::include_proto!("audiostream");
}
use audiostream::audio_streamer_server::{AudioStreamer, AudioStreamerServer};
use audiostream::{EmptyRequest, Track};
use rodio::{Decoder, Source};

use std::{fs::File, io::BufReader};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug)]
pub struct AudioStreamerService {
    track_buffer: Vec<f32>,
    frequency: u32,
    channels: u32,
}

#[tonic::async_trait]
impl AudioStreamer for AudioStreamerService {
    type StreamTrackStream = ReceiverStream<Result<Track, Status>>;

    async fn get_track(&self, _: Request<EmptyRequest>) -> Result<Response<Track>, Status> {
        unimplemented!()
        // return Ok(Response::new(Track {
        //     track_byte: self.track_byte.clone(),
        // }));
    }

    async fn stream_track(
        &self,
        _: Request<EmptyRequest>,
    ) -> Result<Response<Self::StreamTrackStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
        //Is there a way to load the audio file during the stream instead of loading the whole file and
        //converting samples in memory. It takes a lot of memory
        let track = self.track_buffer.clone();
        let channels = self.channels.clone();
        let frequency = self.frequency.clone();

        let length = track.len();
        tokio::spawn(async move {
            let mut i = 0;
            println! {"full length: {:?}", length}
            for byte in &track[..] {
                if i % 10000 == 0 {
                    println!("progress: {} out of {}", i, length)
                }
                //Shouldn't send frequency and channels everytime
                //It streams every single f32 sample which is very slow
                tx.send(Ok(Track {
                    frequency,
                    channels,
                    track_byte: byte.clone(),
                }))
                .await
                .unwrap();
                i += 1;
            }
        });

        return Ok(Response::new(ReceiverStream::new(rx)));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let track_file: BufReader<File> =
        BufReader::new(File::open("./tracks/Jengi_Happy.mp3").unwrap());
    let track_buffer = Decoder::new(track_file).unwrap();
    //Is there a way to load this during the stream instead of loading the whole file and
    //converting samples in the memory. It takes a lot of memory
    let samples = track_buffer.convert_samples();
    let frequency = samples.sample_rate();
    let channels = samples.channels() as u32;
    println!("collecting samples -- takes a moment");
    let data: Vec<f32> = samples.collect();
    println!("ready");

    let audio_streamer = AudioStreamerService {
        frequency,
        channels,
        track_buffer: data,
    };

    Server::builder()
        .add_service(AudioStreamerServer::new(audio_streamer))
        .serve(addr)
        .await?;

    return Ok(());
}
