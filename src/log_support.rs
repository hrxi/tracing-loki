use serde::ser::SerializeMap;
use serde::Serialize;
use serde::Serializer;
use std::collections::HashMap;
use std::error;
use std::fmt;
use tracing_core::field::Visit;
use tracing_core::Event;
use tracing_core::Field;
use tracing_serde::SerdeMapVisitor;

use crate::labels::ValidatedLabel;

pub struct SerializeEventFieldMapStrippingLogAndKeys<'a>(pub &'a Event<'a>, pub&'a HashMap<String, ValidatedLabel>);

impl<'a> Serialize for SerializeEventFieldMapStrippingLogAndKeys<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = self.0.fields().count();
        let serializer = serializer.serialize_map(Some(len))?;
        let mut visitor = SerdeMapVisitorStrippingLogAndKeys::new(serializer, self.1);
        self.0.record(&mut visitor);
        visitor.finish()
    }
}

struct SerdeMapVisitorStrippingLogAndKeys<'a, S: SerializeMap>(SerdeMapVisitor<S>, &'a HashMap<String, ValidatedLabel>);

impl<'a, S: SerializeMap> SerdeMapVisitorStrippingLogAndKeys<'a, S> {
    fn new(serializer: S, strip_keys: &'a HashMap<String, ValidatedLabel>) -> Self {
        Self(SerdeMapVisitor::new(serializer), strip_keys)
    }
    fn ignore(&self, field: &Field) -> bool {
        field.name().starts_with("log.") || self.1.contains_key(field.name())
    }
    fn finish(self) -> Result<S::Ok, S::Error> {
        self.0.finish()
    }
}

impl<'a, S: SerializeMap> Visit for SerdeMapVisitorStrippingLogAndKeys<'a, S> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if !self.ignore(field) {
            self.0.record_debug(field, value);
        }
    }
    fn record_f64(&mut self, field: &Field, value: f64) {
        if !self.ignore(field) {
            self.0.record_f64(field, value);
        }
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        if !self.ignore(field) {
            self.0.record_i64(field, value);
        }
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        if !self.ignore(field) {
            self.0.record_u64(field, value);
        }
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        if !self.ignore(field) {
            self.0.record_bool(field, value);
        }
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        if !self.ignore(field) {
            self.0.record_str(field, value);
        }
    }
    fn record_error(&mut self, field: &Field, value: &(dyn error::Error + 'static)) {
        if !self.ignore(field) {
            self.0.record_error(field, value);
        }
    }
}
