use rodio::Sink;
use rodio::{OutputStream, OutputStreamBuilder};
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Clone)]
pub struct Sound {
    bytes: Vec<u8>,
}

pub struct AudioSystem {
    active_sounds: HashMap<String, Sound>,
    audio_player: AudioPlayer,
}

impl AudioSystem {
    pub fn new() -> Self {
        AudioSystem {
            audio_player: AudioPlayer::new(),
            active_sounds: HashMap::new(),
        }
    }

    pub fn play_sound(&mut self, sound: &str) {
        if self.audio_player.is_playing(sound) {
            return;
        }

        self.audio_player.play_sound(sound);
    }

    pub async fn load_sounds(&mut self) {
        let bonk_sound = Sound {
            bytes: hydrox_utils::load_binary("bonk.wav")
                .await
                .expect("Audio file should exist"), // Is it really just this easy? what about other file formats? Need a decoder? https://github.com/eshaz/wasm-audio-decoders/tree/master? wav(or pcm) is raw. probably want to use flac if we want lossless compression (smaller files without fidelity loss). other formats SHOULD require decoding. though i think mp3 just worked..
        };

        let mut sounds = HashMap::new();
        sounds.insert("bonk".to_string(), bonk_sound);
        self.active_sounds = sounds;
        self.audio_player.load_sounds(&self.active_sounds);
    }

    pub async fn load_sound(&mut self, sound_file: &str) {
        let audio_binary = Sound {
            bytes: hydrox_utils::load_binary(sound_file)
                .await
                .expect("Audio file should exist"),
        };
        let mut sounds = HashMap::new();
        sounds.insert(sound_file.to_string(), audio_binary);
        self.active_sounds = sounds;
        self.audio_player.load_sounds(&self.active_sounds);
    }
}

struct AudioResource {
    sound_bytes: Sound,
    sink: Option<Sink>,
}

struct AudioPlayer {
    audio_stream: OutputStream,
    audio_resources: HashMap<String, AudioResource>,
}
impl AudioPlayer {
    pub fn new() -> Self {
        let audio_resources = HashMap::new();
        let mut audio_stream =
            OutputStreamBuilder::open_default_stream().expect("There should be an audio stream");
        audio_stream.log_on_drop(false);

        AudioPlayer {
            audio_stream,
            audio_resources,
        }
    }

    pub fn load_sounds(&mut self, sounds: &HashMap<String, Sound>) {
        for (sound_name, sound) in sounds {
            let sink = None;
            let audio_resource = AudioResource {
                sound_bytes: sound.clone(),
                sink,
            };
            self.audio_resources
                .insert(sound_name.to_owned(), audio_resource);
        }
    }

    pub fn is_playing(&self, sound: &str) -> bool {
        let audio_resource = self.audio_resources.get(sound);
        audio_resource.is_some()
            && audio_resource.unwrap().sink.is_some()
            && !audio_resource.unwrap().sink.as_ref().unwrap().empty()
    }

    pub fn play_sound(&mut self, sound: &str) {
        let audio_resource = self.audio_resources.get_mut(sound).unwrap();
        let audio_cursor = Cursor::new(audio_resource.sound_bytes.bytes.clone());
        let sink = rodio::play(self.audio_stream.mixer(), audio_cursor);
        audio_resource.sink = Some(sink.unwrap());
    }
}
