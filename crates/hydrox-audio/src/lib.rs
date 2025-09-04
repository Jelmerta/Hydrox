use rodio::Sink;
use rodio::{OutputStream, OutputStreamBuilder};
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Clone)]
pub struct Sound {
    pub name: String,
    pub bytes: Vec<u8>,
}

pub struct AudioSystem {
    active_sounds: HashMap<String, Sound>,
    audio_player: Option<AudioPlayer>,
    is_loaded: bool,
}

impl AudioSystem {
    pub fn new() -> Self {
        AudioSystem {
            audio_player: Some(AudioPlayer::new()),
            active_sounds: HashMap::new(),
            is_loaded: true,
        }
    }

    pub fn new_load_later() -> Self {
        AudioSystem {
            audio_player: None,
            active_sounds: HashMap::new(),
            is_loaded: false,
        }
    }

    pub fn load(&mut self) {
        if self.is_loaded {
            panic!("Audio system is already loaded");
        }

        self.audio_player = Some(AudioPlayer::new());
        self.is_loaded = true;
    }

    pub fn play_sound(&mut self, sound_name: &str) {
        if !self.is_loaded {
            panic!("Audio system is not loaded");
        }
        let audio_player = self
            .audio_player
            .as_mut()
            .expect("audio player should be available when playing sounds");

        if audio_player.is_playing(sound_name) {
            return;
        }

        let sound = self.active_sounds.get_mut(sound_name);
        if sound.is_none() {
            // Audio apparently not yet loaded... Maybe better handling. Very unlikely to happen: Gesture happened but audio not yet loaded
            return;
        }
        audio_player.play_sound(sound.expect("Sound is loaded"));
    }

    pub fn load_sound(&mut self, sound_name: &str, sound: &Sound) {
        self.active_sounds
            .insert(sound_name.to_string(), sound.clone());
        self.audio_player.as_mut().unwrap().load_sound(&sound);
    }
}

struct AudioResource {
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

    pub fn load_sound(&mut self, sound: &Sound) {
        let audio_resource = AudioResource {
            // sound_bytes: sound.clone(),
            sink: None,
        };
        self.audio_resources
            .insert(sound.name.to_owned(), audio_resource);
    }

    pub fn is_playing(&self, sound: &str) -> bool {
        let audio_resource = self.audio_resources.get(sound);
        audio_resource.is_some()
            && audio_resource.unwrap().sink.is_some()
            && !audio_resource.unwrap().sink.as_ref().unwrap().empty()
    }

    pub fn play_sound(&mut self, sound: &Sound) {
        let audio_resource = self.audio_resources.get_mut(&sound.name).unwrap();
        let audio_cursor = Cursor::new(sound.bytes.clone());
        let sink = rodio::play(self.audio_stream.mixer(), audio_cursor);
        audio_resource.sink = Some(sink.unwrap());
    }
}
