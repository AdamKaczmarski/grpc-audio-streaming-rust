## Objectvies

- Learn gRPC
- Stream music from server to client
- Client requests a song and plays as the bytes are streamed from the server


## gRPC streamer objetives and steps to complete the project

- [x] Read the MP3 files in the directory
- [x] have an endpoint to list them
- [x] client requests the list
- [x] client requests a song

NOTE: I think I should dig deeper here how audio is being streamed, samples collected etc.
As I found it pretty hard to stream raw mp3 file. Are companies storing sampled tracks and just providing sample bytes?
- [ ] server sends the song in a normal manner (without collecting samples etc)

I think buffering could be done with 2 threads (or channels), one thread that downloads new bytes
and adds them to a shared vector that the second thread uses to play the music.
- [ ] buffering the song (downloading only chunks of data | downloading the chunks as the song is playing)
