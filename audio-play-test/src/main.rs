use std::{fs::File, io::BufReader};

use rodio::{Decoder, OutputStream, Sink};

fn main() {
    let (_stream, stream_handler) =
        OutputStream::try_default().expect("Couln't obtain default playback device");
    let sink: Sink = Sink::try_new(&stream_handler).unwrap();
    let track: BufReader<File> = BufReader::new(File::open("./tracks/Jengi_Happy.mp3").unwrap());
    let track_source = Decoder::new(track).unwrap();
    sink.append(track_source);
    sink.set_volume(0.3);
    sink.sleep_until_end();
}
