use crate::sync::OnceCell;
use std::fmt;

/// A thread-safe event that can be used to notify multiple tasks.
///
/// Contrary to [`Notify`](crate::sync::Notify), the event keeps an internal state
/// that can be accessed with [`Event::is_set`].
/// The event can be awaited, and if it is already set, [`Event::wait`] returns
/// immediately.
///
/// # Examples
///
/// ```
/// use tokio::sync::Event;
///
/// static SHUTDOWN: Event = Event::const_new();
///
/// async fn task() {
///     loop {
///         tokio::select! {
///             _ = SHUTDOWN.wait() => return,
///             _ = std::future::pending::<()>() => { /* ... */ }
///         }
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let task = tokio::spawn(task());
///     SHUTDOWN.set();
///     task.await.unwrap();
/// }
/// ```
///
#[derive(Default)]
pub struct Event(OnceCell<()>);

impl Event {
    /// Creates a new `Event` instance.
    #[track_caller]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new empty `Event` instance.
    ///
    /// Equivalent to `Event::new`, except that it can be used in static
    /// variables.
    ///
    /// When using the `tracing` [unstable feature], an `Event` created with
    /// `const_new` will not be instrumented. As such, it will not be visible
    /// in [`tokio-console`]. Instead, [`Event::new`] should be used to
    /// create an instrumented object if that is needed.
    ///
    ///
    /// [`tokio-console`]: https://github.com/tokio-rs/console
    /// [unstable feature]: crate#unstable-features
    #[cfg(not(all(loom, test)))]
    pub fn const_new() -> Self {
        Self(OnceCell::const_new())
    }

    /// Returns if the `Event` is set.
    pub fn is_set(&self) -> bool {
        self.0.initialized()
    }

    /// Set the event.
    ///
    /// All tasks waiting for event to be set will be awakened.
    pub fn set(&self) {
        let _ = self.0.set(());
    }

    /// Wait until the event is set, returning immediately if it is already set.
    ///
    /// If the event is set after the returned future is polled at least once,
    /// then the future is guaranteed to complete, even if the event is unset.
    pub async fn wait(&self) {
        self.0.wait_initialized().await;
    }

    /// Unset the event.
    pub fn unset(&self) {
        self.0.unset();
    }
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event")
            .field("is_set", &self.is_set())
            .finish()
    }
}
