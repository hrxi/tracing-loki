use super::event_channel;
use super::BackgroundTask;
use super::BackgroundTaskController;
use super::Error;
use super::ErrorI;
use super::FormattedLabels;
use super::Layer;
use std::collections::hash_map;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

const DEFAULT_BACKGROUD_TASK_BACKOFF: u64 = 500;
const DEFAULT_CHANNEL_CAP: usize = 512;

/// Create a [`Builder`] for constructing a [`Layer`] and its corresponding
/// [`BackgroundTask`].
///
/// See the crate's root documentation for an example.
pub fn builder() -> Builder {
    let mut http_headers = reqwest::header::HeaderMap::new();
    http_headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/x-snappy"),
    );
    Builder {
        labels: FormattedLabels::new(),
        extra_fields: HashMap::new(),
        http_headers,
        backoff: Duration::from_millis(DEFAULT_BACKGROUD_TASK_BACKOFF),
        channel_cap: DEFAULT_CHANNEL_CAP,
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
    http_headers: reqwest::header::HeaderMap,
    backoff: Duration,
    channel_cap: usize,
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
    pub fn label<S: Into<String>, T: AsRef<str>>(
        mut self,
        key: S,
        value: T,
    ) -> Result<Builder, Error> {
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
    pub fn extra_field<S: Into<String>, T: Into<String>>(
        mut self,
        key: S,
        value: T,
    ) -> Result<Builder, Error> {
        match self.extra_fields.entry(key.into()) {
            hash_map::Entry::Occupied(o) => {
                return Err(Error(ErrorI::DuplicateExtraField(o.key().clone())));
            }
            hash_map::Entry::Vacant(v) => {
                v.insert(value.into());
            }
        }
        Ok(self)
    }
    /// Set an extra HTTP header to be sent with all requests sent to Loki.
    ///
    /// This can be useful to set the `X-Scope-OrgID` header which Loki
    /// processes as the tenant ID in a multi-tenant setup.
    ///
    /// # Example
    ///
    /// ```
    /// # use tracing_loki::Error;
    /// # fn main() -> Result<(), Error> {
    /// let builder = tracing_loki::builder()
    ///     // Set the tenant ID for Loki.
    ///     .http_header("X-Scope-OrgID", "7662a206-fa0f-407f-abe9-261d652c750b")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn http_header<S: AsRef<str>, T: AsRef<str>>(
        mut self,
        key: S,
        value: T,
    ) -> Result<Builder, Error> {
        let key = key.as_ref();
        let value = value.as_ref();
        if self
            .http_headers
            .insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes())
                    .map_err(|_| Error(ErrorI::InvalidHttpHeaderName(key.into())))?,
                reqwest::header::HeaderValue::from_str(value)
                    .map_err(|_| Error(ErrorI::InvalidHttpHeaderValue(key.into())))?,
            )
            .is_some()
        {
            return Err(Error(ErrorI::DuplicateHttpHeader(key.into())));
        }
        Ok(self)
    }

    /// Set the backoff used by the backgroud process.
    ///
    /// # Example
    ///
    /// ```
    /// # use tracing_loki::Error;
    /// # use std::time::Duration;
    /// # fn main() -> Result<(), Error> {
    /// let builder = tracing_loki::builder()
    ///     // Set the period of pushing to Loki.
    ///     .backoff(Duration::from_millis(100));
    /// # Ok(())
    /// # }
    /// ```
    pub fn backoff(mut self, backoff: Duration) -> Builder {
        self.backoff = backoff;
        self
    }

    /// Set the size of the internal event channel.
    /// This has an impact on RAM usage.
    ///
    /// # Example
    ///
    /// ```
    /// # use tracing_loki::Error;
    /// # use std::time::Duration;
    /// # fn main() -> Result<(), Error> {
    /// let builder = tracing_loki::builder()
    ///     // Set the period of pushing to Loki.
    ///     .channel_cap(1024);
    /// # Ok(())
    /// # }
    /// ```
    pub fn channel_cap(mut self, channel_cap: usize) -> Builder {
        self.channel_cap = channel_cap;
        self
    }

    /// Build the tracing [`Layer`] and its corresponding [`BackgroundTask`].
    ///
    /// The `loki_url` is the URL of the Loki server, like
    /// `https://127.0.0.1:3100`.
    ///
    /// The [`Layer`] needs to be registered with a
    /// [`tracing_subscriber::Registry`], and the [`BackgroundTask`] needs to
    /// be [`tokio::spawn`]ed.
    ///
    /// **Note** that unlike the [`layer`](`crate::layer`) function, this
    /// function **does not strip off** the path component of `loki_url` before
    /// appending `/loki/api/v1/push`.
    ///
    /// See the crate's root documentation for an example.
    pub fn build_url(self, loki_url: Url) -> Result<(Layer, BackgroundTask), Error> {
        let (sender, receiver) = event_channel(self.channel_cap);
        Ok((
            Layer {
                sender,
                extra_fields: self.extra_fields,
            },
            BackgroundTask::new(
                loki_url,
                self.http_headers,
                receiver,
                &self.labels,
                self.backoff,
            )?,
        ))
    }
    /// Build the tracing [`Layer`], [`BackgroundTask`] and its
    /// [`BackgroundTaskController`].
    ///
    /// The [`BackgroundTaskController`] can be used to signal the background
    /// task to shut down.
    ///
    /// The `loki_url` is the URL of the Loki server, like
    /// `https://127.0.0.1:3100`.
    ///
    /// The [`Layer`] needs to be registered with a
    /// [`tracing_subscriber::Registry`], and the [`BackgroundTask`] needs to
    /// be [`tokio::spawn`]ed.
    ///
    /// **Note** that unlike the [`layer`](`crate::layer`) function, this
    /// function **does not strip off** the path component of `loki_url` before
    /// appending `/loki/api/v1/push`.
    ///
    /// See the crate's root documentation for an example.
    pub fn build_controller_url(
        self,
        loki_url: Url,
    ) -> Result<(Layer, BackgroundTaskController, BackgroundTask), Error> {
        let (sender, receiver) = event_channel(self.channel_cap);
        Ok((
            Layer {
                sender: sender.clone(),
                extra_fields: self.extra_fields,
            },
            BackgroundTaskController { sender },
            BackgroundTask::new(
                loki_url,
                self.http_headers,
                receiver,
                &self.labels,
                self.backoff,
            )?,
        ))
    }
}
