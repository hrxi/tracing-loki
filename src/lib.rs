use loki_api::logproto as loki;
use loki_api::prost;
use serde::Serialize;
use std::cmp;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fmt::Write as _;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;
use std::time::SystemTime;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tracing::instrument::WithSubscriber;
use tracing_core::field::Field;
use tracing_core::field::Visit;
use tracing_core::span::Attributes;
use tracing_core::span::Id;
use tracing_core::span::Record;
use tracing_core::Event;
use tracing_core::Level;
use tracing_core::Subscriber;
use tracing_log::NormalizeEvent;
use tracing_subscriber::layer::Context as TracingContext;
use tracing_subscriber::registry::LookupSpan;
use url::Url;

use level_map::LevelMap;
use log_support::SerializeEventFieldMapStrippingLog;
use no_subscriber::NoSubscriber;

mod level_map;
mod log_support;
mod no_subscriber;

#[derive(Debug)]
pub enum Error {
    ReservedLabelLevel,
    InvalidLabelCharacter(char),
    InvalidLokiUrl,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match self {
            ReservedLabelLevel => write!(f, "cannot add custom label for `level`"),
            InvalidLabelCharacter(c) => write!(f, "invalid label character: {:?}", c),
            InvalidLokiUrl => write!(f, "invalid Loki URL"),
        }
    }
}
impl error::Error for Error {}

pub fn layer(
    loki_url: Url,
    labels: &HashMap<String, String>,
) -> Result<(Layer, BackgroundTask), Error> {
    let _default_guard = tracing::subscriber::set_default(NoSubscriber::default());
    let (sender, receiver) = mpsc::channel(512);
    Ok((
        Layer { sender },
        BackgroundTask::new(loki_url, receiver, &mut labels.clone())?,
    ))
}

pub struct Layer {
    sender: mpsc::Sender<LokiEvent>,
}

struct LokiEvent {
    trigger_send: bool,
    timestamp: SystemTime,
    level: Level,
    message: String,
}

#[derive(Serialize)]
struct SerializedEvent<'a> {
    #[serde(flatten)]
    event: SerializeEventFieldMapStrippingLog<'a>,
    _spans: &'a [&'a str],
    _target: &'a str,
    _module_path: Option<&'a str>,
    _file: Option<&'a str>,
    _line: Option<u32>,
}

#[derive(Default)]
struct Fields {
    fields: serde_json::Map<String, serde_json::Value>,
}

impl Fields {
    fn record_impl(&mut self, field: &Field, value: serde_json::Value) {
        self.fields.insert(field.name().into(), value);
    }
    fn record<T: Into<serde_json::Value>>(&mut self, field: &Field, value: T) {
        self.record_impl(field, value.into());
    }
}

impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.record(field, format!("{:?}", value));
    }
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.record(field, value);
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record(field, value);
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record(field, value);
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.record(field, value);
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        self.record(field, value);
    }
    fn record_error(&mut self, field: &Field, value: &(dyn error::Error + 'static)) {
        self.record(field, format!("{}", value));
    }
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> tracing_subscriber::Layer<S> for Layer {
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: TracingContext<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();
        if extensions.get_mut::<Fields>().is_none() {
            let mut fields = Fields::default();
            attrs.record(&mut fields);
            extensions.insert(fields);
        }
    }
    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: TracingContext<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();
        let fields = extensions.get_mut::<Fields>().expect("unregistered span");
        values.record(fields);
    }
    fn on_event(&self, event: &Event<'_>, ctx: TracingContext<'_, S>) {
        let timestamp = SystemTime::now();
        let normalized_meta = event.normalized_metadata();
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());
        let mut span_fields: serde_json::Map<String, serde_json::Value> = Default::default();
        let spans = ctx
            .current_span()
            .id()
            .and_then(|id| {
                ctx.span_scope(id).map(|scope| {
                    scope.from_root().fold(Vec::new(), |mut spans, span| {
                        span_fields.extend(
                            span.extensions()
                                .get::<Fields>()
                                .expect("unregistered span")
                                .fields
                                .iter()
                                .map(|(f, v)| (f.clone(), v.clone())),
                        );
                        spans.push(span.name());
                        spans
                    })
                })
            })
            .unwrap_or(Vec::new());
        // TODO: Anything useful to do when the capacity has been reached?
        let _ = self.sender.try_send(LokiEvent {
            trigger_send: !meta.target().starts_with("tracing_loki"),
            timestamp,
            level: *meta.level(),
            message: serde_json::to_string(&SerializedEvent {
                event: SerializeEventFieldMapStrippingLog(&event),
                _spans: &spans,
                _target: meta.target(),
                _module_path: meta.module_path(),
                _file: meta.file(),
                _line: meta.line(),
            })
            .expect("json serialization shouldn't fail"),
        });
    }
}

struct SendQueue {
    encoded_labels: String,
    sending: Vec<LokiEvent>,
    to_send: Vec<LokiEvent>,
}

impl SendQueue {
    fn new(encoded_labels: String) -> SendQueue {
        SendQueue {
            encoded_labels,
            sending: Vec::new(),
            to_send: Vec::new(),
        }
    }
    fn push(&mut self, event: LokiEvent) {
        // TODO: Add limit.
        self.to_send.push(event);
    }
    fn drop_outstanding(&mut self) -> usize {
        let len = self.sending.len();
        self.sending.clear();
        len
    }
    fn on_send_result(&mut self, result: Result<(), ()>) {
        match result {
            Ok(()) => self.sending.clear(),
            Err(()) => {
                self.sending.extend(self.to_send.drain(..));
                mem::swap(&mut self.sending, &mut self.to_send);
            }
        }
    }
    fn should_send(&self) -> bool {
        self.to_send.iter().any(|e| e.trigger_send)
    }
    fn prepare_sending(&mut self) -> loki::StreamAdapter {
        if !self.sending.is_empty() {
            panic!("can only prepare sending while no request is in flight");
        }
        mem::swap(&mut self.sending, &mut self.to_send);
        loki::StreamAdapter {
            labels: self.encoded_labels.clone(),
            entries: self
                .sending
                .iter()
                .map(|e| loki::EntryAdapter {
                    timestamp: Some(e.timestamp.into()),
                    line: e.message.clone(),
                })
                .collect(),
            // Couldn't find documentation except for the promtail source code:
            // https://github.com/grafana/loki/blob/8c06c546ab15a568f255461f10318dae37e022d3/clients/pkg/promtail/client/batch.go#L55-L58
            //
            // In the Go code, the hash value isn't initialized explicitly,
            // hence it is set to 0.
            hash: 0,
        }
    }
}

pub struct BackgroundTask {
    loki_url: Url,
    receiver: ReceiverStream<LokiEvent>,
    queues: LevelMap<SendQueue>,
    buffer: Buffer,
    http_client: reqwest::Client,
    backoff_count: u32,
    backoff: Option<Pin<Box<tokio::time::Sleep>>>,
    send_task:
        Option<Pin<Box<dyn Future<Output = Result<(), Box<dyn error::Error>>> + Send + 'static>>>,
}

impl BackgroundTask {
    fn new(
        loki_url: Url,
        receiver: mpsc::Receiver<LokiEvent>,
        labels: &mut HashMap<String, String>,
    ) -> Result<BackgroundTask, Error> {
        fn level_str(level: Level) -> &'static str {
            match level {
                Level::TRACE => "debug",
                Level::DEBUG => "informational",
                Level::INFO => "notice",
                Level::WARN => "warning",
                Level::ERROR => "error",
            }
        }

        if labels.contains_key("label") {
            return Err(Error::ReservedLabelLevel);
        }
        Ok(BackgroundTask {
            receiver: ReceiverStream::new(receiver),
            loki_url: loki_url
                .join("/loki/api/v1/push")
                .map_err(|_| Error::InvalidLokiUrl)?,
            queues: LevelMap::try_from_fn(|level| {
                labels.insert("level".into(), level_str(level).into());
                let labels_encoded = labels_to_string(&labels)?;
                labels.remove("level");
                Ok(SendQueue::new(labels_encoded))
            })?,
            buffer: Buffer::new(),
            http_client: reqwest::Client::builder()
                .user_agent(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION")
                ))
                .build()
                .expect("reqwest client builder"),
            backoff_count: 0,
            backoff: None,
            send_task: None,
        })
    }
    fn backoff_time(&self) -> (bool, Duration) {
        let backoff_count: u64 = self.backoff_count.into();
        let backoff_time = if backoff_count >= 1 {
            Duration::from_millis(500 * (1 << (backoff_count - 1)))
        } else {
            Duration::from_millis(0)
        };
        (
            backoff_time >= Duration::from_secs(30),
            cmp::min(backoff_time, Duration::from_secs(600)),
        )
    }
}

impl Future for BackgroundTask {
    type Output = ();
    fn poll(mut self: Pin<&mut BackgroundTask>, cx: &mut Context<'_>) -> Poll<()> {
        let mut default_guard = tracing::subscriber::set_default(NoSubscriber::default());
        let mut receiver_done = false;
        while let Poll::Ready(maybe_item) = Pin::new(&mut self.receiver).poll_next(cx) {
            match maybe_item {
                Some(item) => {
                    self.queues[item.level].push(item);
                }
                None => receiver_done = true,
            }
        }
        let mut backing_off = if let Some(backoff) = &mut self.backoff {
            matches!(Pin::new(backoff).poll(cx), Poll::Pending)
        } else {
            false
        };
        if !backing_off {
            self.backoff = None;
        }
        let mut send_task_done;
        loop {
            send_task_done = false;
            if let Some(send_task) = &mut self.send_task {
                match Pin::new(send_task).poll(cx) {
                    Poll::Ready(res) => {
                        if let Err(e) = &res {
                            let (drop_outstanding, backoff_time) = self.backoff_time();
                            drop(default_guard);
                            tracing::error!(
                                error_count = self.backoff_count + 1,
                                ?backoff_time,
                                error = %e,
                                "couldn't send logs to loki",
                            );
                            default_guard =
                                tracing::subscriber::set_default(NoSubscriber::default());
                            if drop_outstanding {
                                let num_dropped: usize =
                                    self.queues.values_mut().map(|q| q.drop_outstanding()).sum();
                                drop(default_guard);
                                tracing::error!(
                                    num_dropped,
                                    "dropped outstanding messages due to sending errors",
                                );
                                default_guard =
                                    tracing::subscriber::set_default(NoSubscriber::default());
                            }
                            self.backoff = Some(Box::pin(tokio::time::sleep(backoff_time)));
                            self.backoff_count += 1;
                            backing_off = true;
                        } else {
                            self.backoff_count = 0;
                        }
                        let res = res.map_err(|_| ());
                        for q in self.queues.values_mut() {
                            q.on_send_result(res);
                        }
                        send_task_done = true;
                    }
                    Poll::Pending => {}
                }
            }
            if send_task_done {
                self.send_task = None;
            }
            if self.send_task.is_none()
                && !backing_off
                && self.queues.values().any(|q| q.should_send())
            {
                let streams = self
                    .queues
                    .values_mut()
                    .map(|q| q.prepare_sending())
                    .filter(|s| !s.entries.is_empty())
                    .collect();
                let body = self
                    .buffer
                    .encode(&loki::PushRequest { streams })
                    .to_owned();
                let request_builder = self.http_client.post(self.loki_url.clone());
                self.send_task = Some(Box::pin(
                    async move {
                        request_builder
                            .header(reqwest::header::CONTENT_TYPE, "application/x-snappy")
                            .body(body)
                            .send()
                            .await?
                            .error_for_status()?;
                        Ok(())
                    }
                    .with_subscriber(NoSubscriber::default()),
                ));
            } else {
                break;
            }
        }
        if receiver_done && send_task_done {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

fn labels_to_string(labels: &HashMap<String, String>) -> Result<String, Error> {
    // Couldn't find documentation except for the promtail source code:
    // https://github.com/grafana/loki/blob/8c06c546ab15a568f255461f10318dae37e022d3/clients/pkg/promtail/client/batch.go#L61-L75
    //
    // Go's %q displays the string in double quotes, escaping a few characters,
    // like Rust's {:?}.
    let mut result = String::new();
    result.push('{');
    for (label, value) in labels {
        // Couldn't find documentation except for the promtail source code:
        // https://github.com/grafana/loki/blob/8c06c546ab15a568f255461f10318dae37e022d3/vendor/github.com/prometheus/prometheus/promql/parser/generated_parser.y#L597-L598
        //
        // Apparently labels that confirm to yacc's "IDENTIFIER" are okay. I couldn't find which
        // those are. Let's be conservative and allow `[A-Za-z_]*`.
        for (i, b) in label.bytes().enumerate() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'_' => {}
                // The first byte outside of the above range must start a UTF-8
                // character.
                _ => {
                    return Err(Error::InvalidLabelCharacter(
                        label[i..].chars().next().unwrap(),
                    ))
                }
            }
        }
        let sep = if result.len() <= 1 { "" } else { "," };
        write!(&mut result, "{}{}={:?}", sep, label, value).unwrap();
    }
    result.push('}');
    Ok(result)
}

struct Buffer {
    encoded: Vec<u8>,
    snappy: Vec<u8>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            encoded: Vec::new(),
            snappy: Vec::new(),
        }
    }
    pub fn encode<'a, T: prost::Message>(&'a mut self, message: &T) -> &'a [u8] {
        self.encoded.clear();
        message
            .encode(&mut self.encoded)
            .expect("protobuf encoding is infallible");
        self.compress_encoded()
    }
    fn compress_encoded(&mut self) -> &[u8] {
        self.snappy
            .resize(snap::raw::max_compress_len(self.encoded.len()), 0);
        // Couldn't find documentation except for the promtail source code:
        // https://github.com/grafana/loki/blob/8c06c546ab15a568f255461f10318dae37e022d3/clients/pkg/promtail/client/batch.go#L101
        //
        // In the Go code, `snappy.Encode` is used, which corresponds to the
        // snappy block format, and not the snappy stream format. hence
        // `snap::raw` instead of `snap::write` is needed.
        let snappy_len = snap::raw::Encoder::new()
            .compress(&self.encoded, &mut self.snappy)
            .expect("snappy encoding is infallible");
        &self.snappy[..snappy_len]
    }
}
