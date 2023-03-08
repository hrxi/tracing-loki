use std::collections::HashMap;
use std::collections::hash_map;
use super::BackgroundTask;
use super::Error;
use super::ErrorI;
use super::FormattedLabels;
use super::Layer;
use super::event_channel;
use url::Url;

/// Create a [`Builder`] for constructing a [`Layer`] and its corresponding
/// [`BackgroundTask`].
///
/// See the crate's root documentation for an example.
pub fn builder() -> Builder {
    Builder {
        labels: FormattedLabels::new(),
        extra_fields: HashMap::new(),
    }
}

/// Builder for constructing a [`Layer`] and its corresponding
/// [`BackgroundTask`].
///
/// See the crate's root documentation for an example.
#[derive(Clone)]
pub struct Builder {
    labels: FormattedLabels,
    extra_fields: HashMap<String, String>,
}

impl Builder {
    /// Add a label to the logs sent to Loki through the built `Layer`.
    ///
    /// Labels are supposed to be closed categories with few possible values.
    /// For example, `"environment"` with values `"ci"`, `"development"`,
    /// `"staging"` or `"production"` would work well.
    ///
    /// For open categories, extra fields are a better fit. See
    /// [`Builder::extra_field`].
    ///
    /// No two labels can share the same name, and the key `"level"` is
    /// reserved for the log level.
    ///
    /// # Errors
    ///
    /// This function will return an error if a key is a duplicate or when the
    /// key is `"level"`.
    ///
    /// # Example
    ///
    /// ```
    /// # use tracing_loki::Error;
    /// # fn main() -> Result<(), Error> {
    /// let builder = tracing_loki::builder()
    ///     .label("environment", "production")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn label<S: Into<String>, T: AsRef<str>>(mut self, key: S, value: T)
        -> Result<Builder, Error>
    {
        self.labels.add(key.into(), value.as_ref())?;
        Ok(self)
    }
    /// Set an extra field that is sent with all log records sent to Loki
    /// through the built layer.
    ///
    /// Fields are meant to be used for open categories or closed categories
    /// with many options. For example, `"run_id"` with randomly generated
    /// [UUIDv4](https://en.wikipedia.org/w/index.php?title=Universally_unique_identifier&oldid=1105876960#Version_4_(random))s
    /// would be a good fit for these extra fields.
    ///
    /// # Example
    ///
    /// ```
    /// # use tracing_loki::Error;
    /// # fn main() -> Result<(), Error> {
    /// let builder = tracing_loki::builder()
    ///     .extra_field("run_id", "5b6aedb4-e2c1-4ad9-b8a7-3ef92b5c8120")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn extra_field<S: Into<String>, T: Into<String>>(mut self, key: S, value: T)
        -> Result<Builder, Error>
    {
        match self.extra_fields.entry(key.into()) {
            hash_map::Entry::Occupied(o) => {
                return Err(Error(ErrorI::DuplicateExtraField(o.key().clone())));
            },
            hash_map::Entry::Vacant(v) => {
                v.insert(value.into());
            },
        }
        Ok(self)
    }
    /// Build the tracing [`Layer`] and its corresponding [`BackgroundTask`].
    ///
    /// The `loki_url` is the URL of the Loki server, like `https://127.0.0.1:3100`.
    ///
    /// The [`Layer`] needs to be registered with a
    /// [`tracing_subscriber::Registry`], and the [`BackgroundTask`] needs to be
    /// [`tokio::spawn`]ed.
    ///
    /// **Note** that unlike the [`layer`](`crate::layer`) function, this
    /// function **does not strip off** the path component of `loki_url` before
    /// appending `/loki/api/v1/push`.
    ///
    /// See the crate's root documentation for an example.
    pub fn build_url(mut self, loki_url: Url) -> Result<(Layer, BackgroundTask), Error> {
        let (sender, receiver) = event_channel();
        Ok((
            Layer {
                sender,
                extra_fields: self.extra_fields,
            },
            BackgroundTask::new(loki_url, receiver, &mut self.labels)?,
        ))
    }
}
