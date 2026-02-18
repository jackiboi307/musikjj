use crate::*;
use std::time::{SystemTime, Duration};

macro_rules! sequence {
    ($root:expr, [$($note:expr $(,)?)*]) => {
        vec![$(
            vec![Note::Midi($root + $note)].into_boxed_slice(),
        )*]
    }
}

// fn create_power_chord(note: Note) -> [Note; 3] {
//     let freq = note.freq();
//     [
//         Note::Freq(freq),
//         Note::Freq(freq * 4.0 / 3.0),
//         Note::Freq(freq * 2.0),
//     ]
// }

pub struct Sequencer {
    pub sequence: Vec<Box<[Note]>>,
    step: usize,
    next_step: SystemTime,
    step_duration: Duration,
}

impl Sequencer {
    pub fn new() -> Self {
        let sequence = sequence!(
            ROOT, [0, 4, 7, 9, 6, 1, 8, 2]
        );

        Self {
            sequence,
            step: 0,
            next_step: SystemTime::UNIX_EPOCH,
            step_duration: Duration::from_secs_f32(60.0 / BPM / 4.0),
        }
    }
}

impl Module for Sequencer {
    fn tick(&mut self) -> Data {
        let now = SystemTime::now();

        if self.next_step <= now {
            // println!("{:?}", now.duration_since(self.next_step).unwrap());

            let length = self.sequence.len();
            if length <= self.step {
                self.step = 0;
            }

            let notes = &self.sequence[self.step];

            self.step = (self.step + 1) % length;
            self.next_step = now + self.step_duration;

            Data::Notes(notes.clone())

        } else {
            Data::Notes(Box::new([]))
        }
    }
    
    define_module! {
        title: "Sequencer",
        output: Notes,
        inputs: [],
    }
}
