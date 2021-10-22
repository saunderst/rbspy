use std::collections::HashMap;
use std::io::Write;
use std::time::SystemTime;

// use crate::core::process::Pid;
use crate::core::types::{StackFrame, StackTrace};

use anyhow::Result;

/*
 * This file contains code to export rbspy profiles in pprof-compatible format.
 *
 * The prost crate (https://crates.io/crates/prost) is used to generate the
 * necessary data structures from the protobuf spec found in the pprof project
 * repo (https://github.com/google/pprof/blob/master/proto/profile.proto).  The
 * file containing the data structures is written to perftools.profiles.rs at
 * build time.
 */
// use prost::Message;

// include!(concat!(env!("OUT_DIR"), "/perftools.profiles.rs"));
pub mod pprofs {
    include!("perftools.profiles.rs");
}
use self::pprofs::{Function, Label, Line, Location, Profile, Sample, ValueType};

#[derive(Default)]
pub struct Stats {
    profile: Profile,
    prev_time: Option<SystemTime>,
    known_frames: HashMap<StackFrame, u64>,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            profile: Profile {
                string_table: vec!["".to_string(), "wall".to_string(), "ms".to_string()], // string indexes 0, 1, 2
                sample_type: vec![ValueType { r#type: 1, unit: 2 }], // 1 and 2 are indexes from string_table
                ..Profile::default()
            },
            ..Stats::default()
        }
    }

    pub fn record(&mut self, stack: &StackTrace) -> Result<()> {
        let mut ms_since_last_sample: i64 = 0;
        let time = stack.time.unwrap_or(SystemTime::now());
        if let Some(prev_time) = self.prev_time {
            ms_since_last_sample = time.duration_since(prev_time)?.as_millis() as i64;
        }
        self.prev_time = Some(time);

        self.add_sample(stack, ms_since_last_sample);

        Ok(())
    }

    fn add_sample(&mut self, stack: &StackTrace, sample_time: i64) {
        let s = Sample {
            location_id: self.location_id(stack),
            value: vec![sample_time],
            label: self.labels(stack),
        };
    }

    fn location_id(&mut self, stack: &StackTrace) -> Vec<u64> {
        let locations = <Vec<u64>>::new();

        for frame in stack.trace {
            locations.push(self.get_location_id(frame));
        }
        locations
    }

    fn get_location_id(&mut self, frame: StackFrame) -> u64 {
        if let Some(id) = self.known_frames.get(&frame) {
            *id
        } else {
            let next_id = std::cmp::max(1, self.known_frames.len()) as u64;
            self.known_frames.insert(frame, next_id);
            self.add_new_location(next_id, frame);
            next_id
        }
    }

    fn add_new_location(&mut self, id: u64, frame: StackFrame) {
        let new_line = Line {
            function_id: self.get_function_id(frame),
            line: frame.lineno as i64,
        };
        let new_location = Location {
            id: id,
            line: vec![new_line],
            ..Location::default()
        };
        self.profile.location.push(new_location);
    }

    fn get_function_id(&self, frame: StackFrame) -> u64 {
        let names = self.profile.string_table;
        let functions = self.profile.function.iter();
        if let Some(function) = functions.find(|f| {
            names[f.name as usize] == frame.name
                && names[f.filename as usize] == frame.relative_path
        }) {
            function.id
        } else {
            let next_id = match functions.map(|f| f.id).max() {
                Some(id) => id + 1,
                None => 1,
            };
            self.add_new_function(next_id, frame);
            next_id
        }
    }

    fn add_new_function(&mut self, id: u64, frame: StackFrame) {
        let new_function = Function {
            id,
            name: self.string_id(frame.name),
            filename: self.string_id(frame.relative_path),
            ..Function::default()
        };
    }

    fn string_id(&mut self, text: String) -> i64 {
        let strings = &mut self.profile.string_table;
        if let Some(id) = strings.iter().position(|s| *s == text) {
            id as i64
        } else {
            let next_id = strings.len() as i64;
            strings.push(text);
            next_id
        }
    }

    fn labels(&mut self, stack: &StackTrace) -> Vec<Label> {
        let mut labels: Vec<Label> = Vec::new();
        if let Some(pid) = stack.pid {
            labels.push(Label {
                key: self.string_id("pid".to_string()),
                num: pid as i64,
                ..Label::default()
            });
        }
        if let Some(thread_id) = stack.thread_id {
            labels.push(Label {
                key: self.string_id("thread_id".to_string()),
                num: thread_id as i64,
                ..Label::default()
            });
        }
        labels
    }

    pub fn write(&self, mut w: &mut dyn Write) -> Result<()> {
        Ok(())
    }
}

/*
    pub struct StackFrame {
        pub name: String,
        pub relative_path: String,
        pub absolute_path: Option<String>,
        pub lineno: u32,
    }

    pub struct StackTrace {
        pub trace: Vec<StackFrame>,
        pub pid: Option<Pid>,
        pub thread_id: Option<usize>,
        pub time: Option<SystemTime>,
    }
*/
