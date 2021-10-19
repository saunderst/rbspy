use std::collections::HashMap;
use std::io::Write;
use std::time::SystemTime;

use crate::core::process::Pid;
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
use prost::Message;

// include!(concat!(env!("OUT_DIR"), "/perftools.profiles.rs"));
pub mod pprofs {
    include!("perftools.profiles.rs");
}
use self::pprofs::{Profile, ValueType, Sample, Location};

#[derive(Default)]
pub struct Stats {
    profile: Profile,
    prev_time: Option<SystemTime>,
    known_frames: HashMap<StackFrame, u64>
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            profile: Profile {
                string_table: vec!["".to_string(), "wall".to_string(), "ms".to_string()], // string indexes 0, 1, 2
                sample_type: vec![ValueType{r#type: 1, unit: 2}],   // 1 and 2 are indexes from string_table
                ..Profile::default()
            },
            ..Stats::default()
        }
    }

    pub fn record(&mut self, stack: &StackTrace) -> Result<()> {
        let ms_since_last_sample:i64 = 0;
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
            label: self.labels(stack)
        };
    }

    fn location_id(&mut self, stack: &StackTrace) -> Vec<u64> {
        let locations = <Vec<u64>>::new();

        for frame in stack.trace {
            locations.push(self.get_a_location_id(frame));
        }
        locations
    }

    fn get_a_location_id(&mut self, frame: StackFrame) -> u64 {
        if let Some(id) = self.known_frames.get(&frame) {
            *id
        } else {
            let next_id= self.known_frames.len() as u64;
            self.known_frames.insert(frame, next_id);
            self.add_new_location(next_id);
            next_id
        }
    }

    fn add_new_location(&mut self, id: u64) {
        let new_location = Location {
            id: next_id,
            line: todo!(),
            is_folded: todo!(),
        }
        self.profile.location
    }
    fn string_id(&self, string_table: Vec<String>) -> u64 {
        
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