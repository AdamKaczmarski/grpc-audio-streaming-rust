pub mod audiostream {
    tonic::include_proto!("audiostream");
}
use audiostream::audio_streamer_server::{AudioStreamer, AudioStreamerServer};
use audiostream::{EmptyRequest, TrackList, TrackRequest, TrackStream};
use prost::bytes::Buf;
use rodio::{Decoder, Source};

use std::fs::DirEntry;
use std::sync::Arc;
use std::{fs::File, io::BufReader};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug)]
pub struct AudioStreamerService {
    // track_buffer: Vec<f32>,
    // frequency: u32,
    // channels: u32,
    tracks: Arc<Vec<DirEntry>>,
}

#[tonic::async_trait]
impl AudioStreamer for AudioStreamerService {
    // Error, message length too large:
    // async fn get_track(&self, _: Request<EmptyRequest>) -> Result<Response<Track>, Status> {
    //     // Track is commented out
    //     // return Ok(Response::new(Track {
    //     //     frequency: self.frequency.clone(),
    //     //     channels: self.channels.clone(),
    //     //     track_bytes: self.track_buffer.clone(),
    //     // }));
    //     return unimplemented!();
    // }
    async fn get_track_list(
        &self,
        _: Request<EmptyRequest>,
    ) -> Result<Response<TrackList>, Status> {
        let mut track_list: TrackList = TrackList::default();

        self.tracks.as_ref().into_iter().for_each(|track| {
            track_list
                .track_names
                .push(track.file_name().clone().into_string().unwrap());
        });

        return Ok(Response::new(track_list));
    }

    type StreamTrackStream = ReceiverStream<Result<TrackStream, Status>>;
    async fn stream_track(
        &self,
        request: Request<TrackRequest>,
    ) -> Result<Response<Self::StreamTrackStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
        let track_name: String = request.into_inner().track_name;

        let track_path = self
            .tracks
            .iter()
            .find(|entry| {
                return entry.file_name().into_string().unwrap() == track_name;
            })
            .unwrap()
            .path();

        let track: BufReader<File> = BufReader::new(File::open(track_path)?);
        let track_buffer = Decoder::new(track).unwrap();

        let samples = track_buffer.convert_samples();
        let frequency = samples.sample_rate();
        let channels = samples.channels() as u32;
        println!("collecting samples -- takes a moment");
        let mut data: Vec<f32> = samples.collect();
        println!("ready");
        // Streaming a single track samples
        //Is there a way to load the audio file during the stream instead of loading the whole file and
        //converting samples in memory. It takes a lot of memory
        // let track = self.track_buffer.clone();
        // let channels = self.channels.clone();
        // let frequency = self.frequency.clone();

        let length = data.len();
        tokio::spawn(async move {
            println! {"full length: {:?}", length}

            let mut chunks_itr = data.chunks_mut(419430);
            let mut ctr: u32 = 0;
            while let Some(chunk) = chunks_itr.next() {
                ctr += 419430;
                tx.send(Ok(TrackStream {
                    frequency,
                    channels,
                    track_byte: chunk.to_vec(),
                    // track_byte: data,
                }))
                .await
                .unwrap();
            }
            println!("sent {}/{} ", ctr, length);
            println!("chinks_itd={:?}", chunks_itr);

            // for byte in &data[..] {
            //     if i % 10000 == 0 {
            //         println!("progress: {} out of {}", i, length)
            //     }
            //     //Shouldn't send frequency and channels everytime
            //     //It streams every single f32 sample which is very slow
            //     tx.send(Ok(TrackStream {
            //         frequency,
            //         channels,
            //         track_byte: byte.clone(),
            //     }))
            //     .await
            //     .unwrap();
            //     i += 1;
            // }
        });

        return Ok(Response::new(ReceiverStream::new(rx)));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    // for entry in std::fs::read_dir("./tracks/")? {
    //     let entry = entry?;
    //     let path = entry.path();
    //     if let Some(extension) = path.extension() {
    //         if extension == "mp3" {
    //             println!("Found mp3 file {:?}", entry.path());
    //         }
    //     } else {
    //         println!("Couldn't get extension {:?}", entry.path());
    //     }
    // }
    let mp3_files: Vec<DirEntry> = std::fs::read_dir("./tracks/")
        .unwrap()
        .into_iter()
        .filter_map(|entry| {
            if let Some(extension) = entry.as_ref().unwrap().path().extension() {
                match extension == "mp3" {
                    true => return Some(entry.unwrap()),
                    false => return None,
                }
            } else {
                return None;
            }
        })
        .collect();
    println!("Found mp3 files={:?}", &mp3_files);

    // let track_file: BufReader<File> =
    //     BufReader::new(File::open("./tracks/Jengi_Happy.mp3").unwrap());
    // let track_buffer = Decoder::new(track_file).unwrap();
    // // //Is there a way to load this during the stream instead of loading the whole file and
    // // //converting samples in the memory. It takes a lot of memory
    // let samples = track_buffer.convert_samples();
    // let frequency = samples.sample_rate();
    // let channels = samples.channels() as u32;
    // // println!("collecting samples -- takes a moment");
    // let data: Vec<f32> = samples.collect();
    // println!("ready");
    //
    let audio_streamer = AudioStreamerService {
        // frequency,
        // channels,
        // track_buffer: data,
        tracks: mp3_files.into(),
    };
    // let track_entries = TracksDirEntries { tracks: mp3_files };

    Server::builder()
        .add_service(AudioStreamerServer::new(audio_streamer))
        .serve(addr)
        .await?;

    return Ok(());
}

// fn find_dir_entry(track_name: &str, dir_entries: &Vec<DirEntry>) -> Option<DirEntry> {
//     let tmp = dir_entries
//         .into_iter()
//         .find(|entry| entry.file_name() == track_name);
//     return tmp;
//     return tmp.unwrap().to_owned();
// }
