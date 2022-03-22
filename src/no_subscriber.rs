//! Copy-pasted from
//! <https://github.com/tokio-rs/tracing/blob/cd12dae73b863d2ad8c3f0d2d5e2c27e8164b7f5/tracing-core/src/dispatcher.rs#L710-L715>.
//!
//! This is done to get a different type ID to work around the issue
//! <https://github.com/tokio-rs/tracing/issues/1999>.

use tracing_core::span;
use tracing_core::Event;
use tracing_core::Interest;
use tracing_core::Metadata;
use tracing_core::Subscriber;

/// A no-op [`Subscriber`].
///
/// [`NoSubscriber`] implements the [`Subscriber`] trait by never being enabled,
/// never being interested in any callsite, and dropping all spans and events.
#[derive(Copy, Clone, Debug, Default)]
pub struct NoSubscriber(());

impl Subscriber for NoSubscriber {
    #[inline]
    fn register_callsite(&self, _: &'static Metadata<'static>) -> Interest {
        Interest::never()
    }

    fn new_span(&self, _: &span::Attributes<'_>) -> span::Id {
        span::Id::from_u64(0xDEAD)
    }

    fn event(&self, _event: &Event<'_>) {}

    fn record(&self, _span: &span::Id, _values: &span::Record<'_>) {}

    fn record_follows_from(&self, _span: &span::Id, _follows: &span::Id) {}

    #[inline]
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        false
    }

    fn enter(&self, _span: &span::Id) {}
    fn exit(&self, _span: &span::Id) {}
}
