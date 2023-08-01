use std::collections::HashSet;
use std::fmt::Write as _;
use tracing_core::Level;

use super::Error;
use super::ErrorI;

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
    pub fn add(&mut self, key: String, value: &str) -> Result<(), Error> {
        // Couldn't find documentation except for the promtail source code:
        // https://github.com/grafana/loki/blob/8c06c546ab15a568f255461f10318dae37e022d3/vendor/github.com/prometheus/prometheus/promql/parser/generated_parser.y#L597-L598
        //
        // Apparently labels that confirm to yacc's "IDENTIFIER" are okay. I
        // couldn't find which those are. Let's be conservative and allow
        // `[A-Za-z_]*`.
        for (i, b) in key.bytes().enumerate() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'_' => {}
                // The first byte outside of the above range must start a UTF-8
                // character.
                _ => {
                    let c = key[i..].chars().next().unwrap();
                    return Err(Error(ErrorI::InvalidLabelCharacter(key, c)));
                }
            }
        }
        if key == "level" {
            return Err(Error(ErrorI::ReservedLabelLevel));
        }

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

#[cfg(test)]
mod test {
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
        assert!(FormattedLabels::new().add("level".into(), "").is_err());
        assert!(FormattedLabels::new().add("level".into(), "blurb").is_err());
    }

    #[test]
    fn duplicate() {
        let mut labels = FormattedLabels::new();
        labels.add("label".into(), "abc").unwrap();
        assert!(labels.clone().add("label".into(), "def").is_err());
        assert!(labels.clone().add("label".into(), "abc").is_err());
        assert!(labels.clone().add("label".into(), "").is_err());
    }
}
