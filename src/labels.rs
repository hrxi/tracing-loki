use std::collections::HashSet;
use std::fmt::Write as _;
use tracing_core::Level;

#[allow(unused)]
use tracing_core::field::Visit;

#[allow(unused)]
use std::collections::HashMap;

use super::Error;
use super::ErrorI;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub struct ValidatedLabel(String);

#[derive(Clone)]
pub struct FormattedLabels {
    seen_keys: HashSet<String>,
    formatted: String,
}

impl FormattedLabels {
    pub fn new() -> FormattedLabels {
        FormattedLabels {
            seen_keys: HashSet::new(),
            formatted: String::from("{"),
        }
    }
    pub fn add(&mut self, ValidatedLabel(key): ValidatedLabel, value: &str) -> Result<(), Error> {
        // Couldn't find documentation except for the promtail source code:
        // https://github.com/grafana/loki/blob/8c06c546ab15a568f255461f10318dae37e022d3/clients/pkg/promtail/client/batch.go#L61-L75
        //
        // Go's %q displays the string in double quotes, escaping a few
        // characters, like Rust's {:?}.
        let old_len = self.formatted.len();
        let sep = if self.formatted.len() <= 1 { "" } else { "," };
        write!(&mut self.formatted, "{}{}={:?}", sep, key, value).unwrap();

        if let Some(duplicate_key) = self.seen_keys.replace(key) {
            self.formatted.truncate(old_len);
            return Err(Error(ErrorI::DuplicateLabel(duplicate_key)));
        }
        Ok(())
    }

    #[cfg(feature = "dynamic-labels")]
    pub fn contains(&self, ValidatedLabel(key): &ValidatedLabel) -> bool {
        self.seen_keys.contains(key)
    }

    #[cfg(feature = "dynamic-labels")]
    /// Join with another set of labels that are already formatted.
    /// Does not check that the other labels are valid or that they don't contain duplicates
    /// including with the current set of labels. That is checked by the builder.
    pub fn join_with_finished(self, other_formatted: String) -> String {
        if self.formatted.len() <= 1 {
            return other_formatted;
        }

        let mut result = self.formatted;
        if result.len() > 1 {
            result.push(',');
        }
        result.push_str(&other_formatted[1..]);
        result
    }
    pub fn finish(&self, level: Level) -> String {
        let mut result = self.formatted.clone();
        if result.len() > 1 {
            result.push(',');
        }
        result.push_str(match level {
            Level::TRACE => "level=\"trace\"}",
            Level::DEBUG => "level=\"debug\"}",
            Level::INFO => "level=\"info\"}",
            Level::WARN => "level=\"warn\"}",
            Level::ERROR => "level=\"error\"}",
        });
        result
    }
}

impl ValidatedLabel {
    pub fn new(label: String) -> Result<Self, Error> {
        // Couldn't find documentation except for the promtail source code:
        // https://github.com/grafana/loki/blob/8c06c546ab15a568f255461f10318dae37e022d3/vendor/github.com/prometheus/prometheus/promql/parser/generated_parser.y#L597-L598
        //
        // Apparently labels that confirm to yacc's "IDENTIFIER" are okay. I
        // couldn't find which those are. Let's be conservative and allow
        // `[A-Za-z_]*`.
        for (i, b) in label.bytes().enumerate() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'_' => {}
                // The first byte outside of the above range must start a UTF-8
                // character.
                _ => {
                    let c = label[i..].chars().next().unwrap();
                    return Err(Error(ErrorI::InvalidLabelCharacter(label, c)));
                }
            }
        }
        if label == "level" {
            return Err(Error(ErrorI::ReservedLabelLevel));
        }
        Ok(ValidatedLabel(label))
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "dynamic-labels")]
#[derive(Clone)]
pub struct LabelSelectorVisitor<'a> {
    select_keys: &'a HashMap<String, ValidatedLabel>,
    found_labels: Vec<(ValidatedLabel, String)>,
}

#[cfg(feature = "dynamic-labels")]
impl<'a> LabelSelectorVisitor<'a> {
    pub fn new(select_keys: &'a HashMap<String, ValidatedLabel>) -> Self {
        Self {
            select_keys,
            found_labels: Vec::new(),
        }
    }

    pub fn finish(mut self, level: Level) -> String {
        self.found_labels.sort_by(|val1, val2| val1.0.cmp(&val2.0));
        let mut labels = FormattedLabels::new();
        for (key, value) in self.found_labels {
            match labels.add(key.to_owned(), &value) {
                Ok(()) | Err(Error(ErrorI::DuplicateLabel(_))) => (), // Ignore duplicate labels.
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }
        labels.finish(level)
    }
}

#[cfg(feature = "dynamic-labels")]
impl<'a> Visit for LabelSelectorVisitor<'a> {
    fn record_debug(&mut self, field: &tracing_core::Field, value: &dyn std::fmt::Debug) {
        if let Some(validated) = self.select_keys.get(field.name()) {
            self.found_labels
                .push((validated.clone(), format!("{:?}", value)));
        }
    }

    fn record_str(&mut self, field: &tracing_core::Field, value: &str) {
        // Overriding this to avoid using the debug implementation that would escape + add quotes twice
        // (including the final formatting).
        if let Some(validated) = self.select_keys.get(field.name()) {
            self.found_labels
                .push((validated.clone(), value.to_owned()));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::labels::ValidatedLabel;

    use super::FormattedLabels;
    use tracing_core::Level;

    #[test]
    fn simple() {
        assert_eq!(
            FormattedLabels::new().finish(Level::TRACE),
            r#"{level="trace"}"#,
        );
        assert_eq!(
            FormattedLabels::new().finish(Level::DEBUG),
            r#"{level="debug"}"#,
        );
        assert_eq!(
            FormattedLabels::new().finish(Level::INFO),
            r#"{level="info"}"#,
        );
        assert_eq!(
            FormattedLabels::new().finish(Level::WARN),
            r#"{level="warn"}"#,
        );
        assert_eq!(
            FormattedLabels::new().finish(Level::ERROR),
            r#"{level="error"}"#,
        );
    }

    #[test]
    fn level() {
        assert!(ValidatedLabel::new("level".into()).is_err());
    }

    #[test]
    fn duplicate() {
        let mut labels = FormattedLabels::new();
        let validated = ValidatedLabel::new("abc".into()).unwrap();
        labels.add(validated.clone(), "abc").unwrap();
        assert!(labels.clone().add(validated.clone(), "def").is_err());
        assert!(labels.clone().add(validated.clone(), "abc").is_err());
        assert!(labels.clone().add(validated.clone(), "").is_err());
    }
}
