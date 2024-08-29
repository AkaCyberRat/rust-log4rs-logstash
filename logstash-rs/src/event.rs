use crate::event::level_serializer::SerializableLevel;
use crate::event::logstash_date_format::SerializableDateTime;
use chrono::{DateTime, SecondsFormat, Utc};
use log::Level;
use serde::ser::SerializeMap;
use serde::Serializer;
use serde_json::Value;
use std::{collections::HashMap, time::SystemTime};

#[derive(Debug, serde::Deserialize, Copy, Clone)]
pub enum TimePrecision {
    /// Format whole seconds only, with no decimal point nor subseconds.
    Secs,

    /// Use fixed 3 subsecond digits.
    Millis,

    /// Use fixed 6 subsecond digits.
    Micros,

    /// Use fixed 9 subsecond digits.
    Nanos,
}

impl From<TimePrecision> for SecondsFormat {
    fn from(val: TimePrecision) -> Self {
        match val {
            TimePrecision::Secs => SecondsFormat::Secs,
            TimePrecision::Millis => SecondsFormat::Millis,
            TimePrecision::Micros => SecondsFormat::Micros,
            TimePrecision::Nanos => SecondsFormat::Nanos,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogStashRecord {
    pub timestamp: DateTime<Utc>,
    pub module: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub level: Level,
    pub target: String,
    pub time_precision: TimePrecision,
    pub fields: HashMap<String, Value>,
}

impl serde::Serialize for LogStashRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(7))?;

        map.serialize_entry(
            "@timestamp",
            &SerializableDateTime::new(self.timestamp, self.time_precision),
        )?;

        if let Some(ref module) = self.module {
            map.serialize_entry("module", module)?;
        }
        if let Some(ref file) = self.file {
            map.serialize_entry("file", file)?;
        }
        if let Some(line) = self.line {
            map.serialize_entry("line", &line)?;
        }

        map.serialize_entry("level", &SerializableLevel::from(self.level))?;
        map.serialize_entry("target", &self.target)?;

        for (key, value) in self.fields.iter() {
            map.serialize_entry(key, value)?;
        }

        map.end()
    }
}

impl LogStashRecord {
    /// Initialize record with current time in `timestamp` field
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            ..Default::default()
        }
    }

    pub fn from_record(record: &log::Record) -> Self {
        let mut event = LogStashRecord::new();
        let meta = record.metadata();

        event.module = record.module_path().map(|p| p.into());
        event.file = record.file().map(|p| p.into());
        event.line = record.line();
        event.level = meta.level();
        event.target = meta.target().into();
        event.time_precision = TimePrecision::Millis;
        event.add_data("message", record.args().to_string().into());
        event
    }

    pub fn set_timestamp(&mut self, timestamp: SystemTime) -> &mut Self {
        self.timestamp = timestamp.into();
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: Value) -> &mut Self {
        self.fields.insert(format!("@metadata.{}", key), value);
        self
    }

    pub fn add_data(&mut self, key: &str, value: Value) -> &mut Self {
        self.fields.insert(key.into(), value);
        self
    }

    pub fn with_data_from_map(mut self, extra_fields: &HashMap<String, Value>) -> Self {
        if !extra_fields.is_empty() {
            self.fields.extend(
                extra_fields
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone())),
            );
        }
        self
    }

    pub fn with_time_precision(mut self, teme_precision: TimePrecision) -> Self {
        self.time_precision = teme_precision;
        self
    }
}

impl Default for LogStashRecord {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            level: Level::Warn,
            time_precision: TimePrecision::Millis,
            module: Default::default(),
            file: Default::default(),
            line: Default::default(),
            target: Default::default(),
            fields: Default::default(),
        }
    }
}

mod logstash_date_format {
    use crate::event::TimePrecision;
    use chrono::{DateTime, Utc};
    use serde::{self, Serializer};

    pub(crate) struct SerializableDateTime {
        date_time: DateTime<Utc>,
        time_precision: TimePrecision,
    }

    impl SerializableDateTime {
        pub fn new(date_time: DateTime<Utc>, time_precision: TimePrecision) -> Self {
            Self {
                date_time,
                time_precision,
            }
        }
    }

    impl serde::Serialize for SerializableDateTime {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = self
                .date_time
                .to_rfc3339_opts(self.time_precision.into(), true);

            serializer.serialize_str(&s)
        }
    }
}

mod level_serializer {
    use log::Level;
    use serde::{self, Serializer};

    pub struct SerializableLevel(Level);

    impl serde::Serialize for SerializableLevel {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(self.0.as_str())
        }
    }

    impl From<Level> for SerializableLevel {
        fn from(value: Level) -> Self {
            Self(value)
        }
    }
}
