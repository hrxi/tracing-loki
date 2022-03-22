use serde::ser::SerializeMap;
use serde::Serialize;
use serde::Serializer;
use std::error;
use std::fmt;
use tracing_core::field::Visit;
use tracing_core::Event;
use tracing_core::Field;
use tracing_serde::SerdeMapVisitor;

pub struct SerializeEventFieldMapStrippingLog<'a>(pub &'a Event<'a>);

impl<'a> Serialize for SerializeEventFieldMapStrippingLog<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = self.0.fields().count();
        let serializer = serializer.serialize_map(Some(len))?;
        let mut visitor = SerdeMapVisitorStrippingLog::new(serializer);
        self.0.record(&mut visitor);
        visitor.finish()
    }
}

struct SerdeMapVisitorStrippingLog<S: SerializeMap>(SerdeMapVisitor<S>);

impl<S: SerializeMap> SerdeMapVisitorStrippingLog<S> {
    fn new(serializer: S) -> SerdeMapVisitorStrippingLog<S> {
        SerdeMapVisitorStrippingLog(SerdeMapVisitor::new(serializer))
    }
    fn ignore(field: &Field) -> bool {
        field.name().starts_with("log.")
    }
    fn finish(self) -> Result<S::Ok, S::Error> {
        self.0.finish()
    }
}

impl<S: SerializeMap> Visit for SerdeMapVisitorStrippingLog<S> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if !Self::ignore(field) {
            self.0.record_debug(field, value);
        }
    }
    fn record_f64(&mut self, field: &Field, value: f64) {
        if !Self::ignore(field) {
            self.0.record_f64(field, value);
        }
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        if !Self::ignore(field) {
            self.0.record_i64(field, value);
        }
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        if !Self::ignore(field) {
            self.0.record_u64(field, value);
        }
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        if !Self::ignore(field) {
            self.0.record_bool(field, value);
        }
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        if !Self::ignore(field) {
            self.0.record_str(field, value);
        }
    }
    fn record_error(&mut self, field: &Field, value: &(dyn error::Error + 'static)) {
        if !Self::ignore(field) {
            self.0.record_error(field, value);
        }
    }
}
