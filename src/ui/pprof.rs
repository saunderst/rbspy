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
    traces: Vec<StackTrace>
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
        let time = stack.time.unwrap_or_else(SystemTime::now);
        if let Some(prev_time) = self.prev_time {
            ms_since_last_sample = time.duration_since(prev_time)?.as_millis() as i64;
        }
        self.prev_time = Some(time);

        self.add_sample(stack, ms_since_last_sample);
        // self.traces.push(stack.to_owned());

        Ok(())
    }

    fn add_sample(&mut self, stack: &StackTrace, sample_time: i64) {
        let s = Sample {
            location_id: self.location_ids(stack),
            value: vec![sample_time],
            label: self.labels(stack),
        };
        self.profile.sample.push(s);
    }

    fn location_ids(&mut self, stack: &StackTrace) -> Vec<u64> {
        let mut ids = <Vec<u64>>::new();

        for frame in &stack.trace {
            ids.push(self.get_or_create_location_id(frame));
        }
        ids
    }

    fn get_or_create_location_id(&mut self, frame: &StackFrame) -> u64 {
        // our lookup table has the arbitrary ids (1..n) we use for location ids
        if let Some(id) = self.known_frames.get(frame) {
            *id
        } else {
            let next_id = self.known_frames.len() as u64 + 1; //ids must be non-0, so start at 1
            self.known_frames.insert(frame.clone(), next_id); // add to our lookup table
            let newloc = self.new_location(next_id, frame); // use the same id for the location table
            self.profile.location.push(newloc);
            next_id
        }
    }

    fn new_location(&mut self, id: u64, frame: &StackFrame) -> Location {
        let new_line = Line {
            function_id: self.get_or_create_function_id(frame),
            line: frame.lineno as i64,
        };
        Location {
            id,
            line: vec![new_line],
            ..Location::default()
        }
    }

    fn get_or_create_function_id(&mut self, frame: &StackFrame) -> u64 {
        let strings = &self.profile.string_table;
        let mut functions = self.profile.function.iter();
        if let Some(function) = functions.find(|f| {
            frame.name == strings[f.name as usize]
                && frame.relative_path == strings[f.filename as usize]
        }) {
            function.id
        } else {
            let functions = self.profile.function.iter();
            let mapped_iter = functions.map(|f| f.id);
            let max_map = mapped_iter.max();
            // let next_id = match functions.map(|f| f.id).max() {
            let next_id = match max_map {
                Some(id) => id + 1,
                None => 1,
            };
            let f = self.new_function(next_id, frame);
            self.profile.function.push(f);
            next_id
        }
    }

    fn new_function(&mut self, id: u64, frame: &StackFrame) -> Function {
        Function {
            id,
            name: self.string_id(&frame.name),
            filename: self.string_id(&frame.relative_path),
            ..Function::default()
        }
    }

    fn string_id(&mut self, text: &str) -> i64 {
        let strings = &mut self.profile.string_table;
        if let Some(id) = strings.iter().position(|s| *s == *text) {
            id as i64
        } else {
            let next_id = strings.len() as i64;
            strings.push((*text).to_owned());
            next_id
        }
    }

    fn labels(&mut self, stack: &StackTrace) -> Vec<Label> {
        let mut labels: Vec<Label> = Vec::new();
        if let Some(pid) = stack.pid {
            labels.push(Label {
                key: self.string_id(&"pid".to_string()),
                num: pid as i64,
                ..Label::default()
            });
        }
        if let Some(thread_id) = stack.thread_id {
            labels.push(Label {
                key: self.string_id(&"thread_id".to_string()),
                num: thread_id as i64,
                ..Label::default()
            });
        }
        labels
    }

    pub fn write(&self, _w: &mut dyn Write) -> Result<()> {
        println!("{:?}", self.profile);
        Ok(())
    }
}