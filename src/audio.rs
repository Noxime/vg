// TODO: Add positional audio (left-right mostly)

extern crate ears;

use self::ears::*;

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{
        mpsc::{channel, Sender},
        Mutex,
    },
    thread::spawn,
};

lazy_static! {
    static ref SENDER: Mutex<Option<Sender<String>>> = Mutex::new(None);
}

pub fn init() {
    let (tx, rx) = channel();
    *SENDER.lock().unwrap() = Some(tx);
    debug!("Started audio thread");
    spawn(move || {
        let mut cache = HashMap::new();
        let mut playing = vec![];
        while let Ok(msg) = rx.recv() {
            debug!("Playing sound {}", msg);
            if !cache.contains_key(&msg) {
                trace!("Added sound to soundcache");
                cache.insert(
                    msg.clone(),
                    Rc::new(RefCell::new(SoundData::new(&msg).unwrap())),
                );
            }
            let sd = cache
                .get(&msg)
                .expect("Where the fuck did our audiodata go");
            let mut s = Sound::new_with_data(sd.clone()).unwrap();
            s.play();
            playing.push(s);
            playing = playing.into_iter().filter(|s| s.is_playing()).collect();
        }
        error!("Audio transmit channel closed!");
    });
}
pub fn play(path: String) {
    if let Some(ref v) = *SENDER.lock().unwrap() {
        if let Err(why) = v.send(path) {
            error!("Failed to send audio: {:?}", why);
        }
    }
}
