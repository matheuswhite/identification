use aule::prelude::{Block, Signal};
use std::{fs::File, path::PathBuf};

pub struct CSVSignal {
    path: PathBuf,
    reader: csv::Reader<File>,
    time_index: usize,
    value_index: usize,
    current_record: Option<Record>,
    last_record: Record,
}

#[derive(Clone)]
struct Record {
    time: f64,
    value: f64,
}

impl CSVSignal {
    pub fn new(path: &str, time_index: usize, value_index: usize) -> Result<Self, csv::Error> {
        let mut obj = Self {
            path: PathBuf::from(path),
            reader: csv::Reader::from_path(path)?,
            time_index,
            value_index,
            current_record: None,
            last_record: Record {
                time: 0.0,
                value: 0.0,
            },
        };

        obj.current_record = obj.next_record();

        Ok(obj)
    }

    fn next_record(&mut self) -> Option<Record> {
        let records = self.reader.records();
        let Some(Ok(rec)) = records.into_iter().next() else {
            return None;
        };

        let time: f64 = rec[self.time_index].parse().unwrap();
        let value: f64 = rec[self.value_index].parse().unwrap();

        let record = Record { time, value };

        Some(record)
    }
}

impl Clone for CSVSignal {
    fn clone(&self) -> Self {
        Self::new(
            self.path.to_str().unwrap(),
            self.time_index,
            self.value_index,
        )
        .unwrap()
    }
}

impl Block for CSVSignal {
    type Input = ();
    type Output = Option<f64>;

    fn output(&mut self, input: Signal<Self::Input>) -> Signal<Self::Output> {
        let Some(record) = self.current_record.clone() else {
            return input.map(|_| None);
        };

        if input.delta.sim_time().as_secs_f64() >= record.time {
            self.last_record = record;
            self.current_record = self.next_record();
        }

        let Some(record) = self.current_record.clone() else {
            return input.map(|_| None);
        };

        let output = input.map(|_| {
            let start = self.last_record.time;
            let end = record.time;

            if end == start {
                return Some(record.value);
            }

            let t = input.delta.sim_time().as_secs_f64();
            let alpha = (t - start) / (end - start);

            let value = self.last_record.value + alpha * (record.value - self.last_record.value);

            Some(value)
        });

        output
    }
}
