use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use std::cmp::Ordering;
#[cfg(target_arch = "wasm32")]
use wee_alloc;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Debug)]
pub struct Tone {
    frequency: u16,
    time: u32,
}

fn note_to_frequency(note: u8) -> u16 {
    (440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)) as u16
}

pub fn convert_midi(midi_file_content: Vec<u8>) -> Vec<u8> {
    let smf = Smf::parse(&midi_file_content).unwrap();
    let ticks_per_beat = if let Timing::Metrical(x) = smf.header.timing {
        x.as_int()
    } else {
        unimplemented!()
    };
    let mut microseconds_per_tick = 0;
    let mut note_events = Vec::new();
    let mut current_tick = 0;
    for event in &smf.tracks[0] {
        current_tick += event.delta.as_int();
        let body = event.kind;
        match body {
            TrackEventKind::Meta(MetaMessage::Tempo(ms_per_beat)) => {
                microseconds_per_tick = ms_per_beat.as_int() / (ticks_per_beat as u32);
            }
            TrackEventKind::Midi {
                message: MidiMessage::NoteOn { key, vel },
                ..
            } => note_events.push((current_tick, key, vel != 0)),
            TrackEventKind::Midi {
                message: MidiMessage::NoteOff { key, .. },
                ..
            } => note_events.push((current_tick, key, false)),
            _ => {}
        }
    }
    note_events.sort_unstable_by(|(tick1, _, start1), (tick2, _, start2)| {
        if tick1 != tick2 {
            tick1.cmp(tick2)
        } else if start1 == start2 {
            Ordering::Equal
        } else if !*start1 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
    let mut result = Vec::with_capacity(note_events.len() / 2);
    let mut current_key = 0xff;
    let mut current_key_start = 0;
    for (current_tick, key, start) in note_events {
        if !start && key == current_key {
            result.push(Tone {
                frequency: note_to_frequency(key.as_int()),
                time: (current_tick - current_key_start) * microseconds_per_tick,
            });
            current_key = 0xff;
            current_key_start = 0;
        } else if start && key < current_key {
            if current_key != 0xff {
                result.push(Tone {
                    frequency: note_to_frequency(key.as_int()),
                    time: (current_tick - current_key_start) * microseconds_per_tick,
                });
            }
            current_key = key.as_int();
            current_key_start = current_tick;
        }
    }
    let result: Vec<_> = result
        .into_iter()
        .flat_map(|Tone { frequency, time }| {
            frequency
                .to_le_bytes()
                .into_iter()
                .chain(time.to_le_bytes().into_iter())
        })
        .collect();
    result
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = convert_midi)]
pub fn convert_midi_wasm(midi_file_content: Vec<u8>) -> Vec<u8> {
    convert_midi(midi_file_content)
}
