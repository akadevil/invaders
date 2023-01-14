use rodio::{
  source::{Buffered, Source},
  Decoder, OutputStream, OutputStreamHandle, Sink,
};
use std::{collections::HashMap};
use std::io::{Cursor, Read};
use std::{fs::File, path::Path};

/// A simple 4-track audio system to load/decode audio files from disk to play later. Supported
/// formats are: MP3, WAV, Vorbis and Flac.
#[derive(Default)]
pub struct Audio {
  clips: HashMap<String, Buffered<Decoder<Cursor<Vec<u8>>>>>,
  channels: Vec<Sink>,
  current_channel: usize,
  output: Option<(OutputStream, OutputStreamHandle)>,
}

impl Audio {
  /// Create a new sound subsystem.  You only need one of these -- you can use it to load and play
  /// any number of audio clips.
  pub fn new() -> Self {
      if let Ok(output) = OutputStream::try_default() {
          let clips = HashMap::new();
          let mut channels: Vec<Sink> = Vec::new();
          for i in 0..4 {
              let sink = Sink::try_new(&output.1)
                  .unwrap_or_else(|_| panic!("Failed to create sound channel {}", i));
              channels.push(sink);
          }
          Self {
              clips,
              channels,
              current_channel: 0,
              output: Some(output),
          }
      } else {
          Self {
              clips: HashMap::new(),
              channels: Vec::new(),
              current_channel: 0,
              output: None,
          }
      }
  }
  /// If no sound device was detected, the audio subsystem will run in a disabled mode that
  /// doesn't actually do anything. This method indicates whether audio is disabled.
  pub fn disabled(&self) -> bool {
    self.output.is_none()
  }

  pub fn file_open<P:AsRef<Path>> (path: P) -> Vec<u8> {
    let mut file_vec: Vec<u8> = Vec::new();
      File::open(path.as_ref())
          .expect("Couldn't find audio file to add.")
          .read_to_end(&mut file_vec)
          .expect("Failed reading in opened audio file.");

    file_vec
  }
  /// Add an audio clip to play.  Audio clips will be decoded and buffered during this call so
  /// the first call to `.play()` is not staticky if you compile in debug mode.  `name` is what
  /// you will refer to this clip as when you need to play it.  Files known to be supported by the
  /// underlying library (rodio) at the time of this writing are MP3, WAV, Vorbis and Flac.
  pub fn add<S: AsRef<str>, P: AsRef<Path>>(&mut self, name: S, path: P) {
    if self.disabled() {
        return;
    }

    let file_vec: Vec<u8> = Audio::file_open(path);
    let cursor = Cursor::new(file_vec);
    match Decoder::new(cursor) {
      Ok(file) => {
        let buffered = file.buffered();
        let warm = buffered.clone();
  
        for i in warm {
            #[allow(clippy::drop_copy)]
            drop(i);
        }
  
        self.clips.insert(name.as_ref().to_string(), buffered);
      },
      Err(error) => panic!("Problem with the file: {:?}", error),
    };
      // Buffers are lazily decoded, which often leads to static on first play on low-end systems
      // or when you compile in debug mode.  Since this library is intended for educational
      // projects, those are going to be common conditions.  So, to optimize for our use-case, we
      // will pre-warm all of our audio buffers by forcing things to be decoded and cached right
      // now when we first load the file.  I would like to find a cleaner way to do this, but the
      // following scheme (iterating through a clone and discarding the decoded frames) works
      // since clones of a Buffered share the actual decoded data buffer cache by means of Arc and
      // Mutex.
  }
  /// Play an audio clip that has already been loaded.  `name` is the name you chose when you
  /// added the clip to the `Audio` system. If you forgot to load the clip first, this will crash.
  pub fn play<S: AsRef<str>>(&mut self, name: S) {
    if self.disabled() {
      return;
    }

    let buffer = self.clips
      .get(name.as_ref())
      .expect("No clip by that name.")
      .clone();
    self.channels[self.current_channel].append(buffer);
    self.current_channel += 1;

    if self.current_channel >= self.channels.len() {
      self.current_channel = 0;
    }
  }

  // pub fn stop(&self) {
  //   // self.channels[self.current_channel].sleep_until_end();
  //   drop(&self.channels[self.current_channel]);
  // }
  /// Block until no sounds are playing. Convenient for keeping a thread alive until all sounds
  /// have played.
  pub fn wait(&self) {
    if self.disabled() {
      return;
    }

    loop {
      if self.channels.iter().any(|x| !x.empty()) {
          std::thread::sleep(std::time::Duration::from_millis(50));
          continue;
      }
      break;
    }
  }
}