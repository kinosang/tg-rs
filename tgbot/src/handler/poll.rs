use crate::{
    api::Api,
    methods::GetUpdates,
    types::{AllowedUpdate, Integer, ResponseError, Update},
};
use failure::Error;
use futures03::{
    compat::{Compat01As03, Future01CompatExt},
    ready,
    task::Context,
    Future, Poll, Stream,
};
use log::error;
use std::{
    cmp::max,
    collections::{HashSet, VecDeque},
    mem,
    pin::Pin,
    time::Duration,
};
use tokio_timer::sleep;

const DEFAULT_LIMIT: Integer = 100;
const DEFAULT_POLL_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_ERROR_TIMEOUT: Duration = Duration::from_secs(5);

type RunningFuture = Pin<Box<Future<Output = Result<Vec<Update>, Error>>>>;

enum State {
    BufferedResults(VecDeque<Update>),
    Running(RunningFuture),
    Idling(Compat01As03<tokio_timer::Delay>),
}

/// Updates stream used for long polling
#[must_use = "streams do nothing unless polled"]
pub struct UpdatesStream {
    api: Api,
    options: UpdatesStreamOptions,
    state: State,
    should_retry: bool,
}

fn make_request(api: &Api, options: &UpdatesStreamOptions) -> RunningFuture {
    let fut = api.execute(
        GetUpdates::default()
            .offset(options.offset + 1)
            .limit(options.limit)
            .timeout(options.poll_timeout)
            .allowed_updates(options.allowed_updates.clone()),
    );
    Box::pin(fut)
}

impl State {
    fn switch_to_idle(&mut self, err: Error) {
        error!(
            "An error has occurred while getting updates: {:?}\n{:?}",
            err,
            err.backtrace()
        );
        let error_timeout = err
            .downcast::<ResponseError>()
            .ok()
            .and_then(|err| {
                err.parameters
                    .and_then(|parameters| parameters.retry_after.map(|count| Duration::from_secs(count as u64)))
            })
            .unwrap_or(DEFAULT_ERROR_TIMEOUT);
        mem::replace(self, State::Idling(sleep(error_timeout).compat()));
    }

    fn switch_to_request(&mut self, api: &Api, options: &UpdatesStreamOptions) {
        let fut = make_request(api, options);
        mem::replace(self, State::Running(fut));
    }

    fn switch_to_buffered(&mut self, items: impl IntoIterator<Item = Update>) {
        mem::replace(self, State::BufferedResults(items.into_iter().collect()));
    }
}

impl Stream for UpdatesStream {
    type Item = Result<Update, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unsafe {
            loop {
                let stream = self.as_mut().get_unchecked_mut();
                let mut state = &mut stream.state;
                match &mut state {
                    State::BufferedResults(buffered) => {
                        if let Some(update) = buffered.pop_front() {
                            stream.options.offset = max(stream.options.offset, update.id);
                            cx.waker().clone().wake();
                            return Poll::Ready(Some(Ok(update)));
                        } else {
                            state.switch_to_request(&stream.api, &stream.options);
                        }
                    }
                    State::Running(request_fut) => match ready!(request_fut.as_mut().poll(cx)) {
                        Ok(items) => state.switch_to_buffered(items),
                        Err(err) => {
                            if stream.should_retry {
                                state.switch_to_idle(err)
                            } else {
                                return Poll::Ready(Some(Err(err)));
                            }
                        }
                    },
                    State::Idling(delay_fut) => {
                        // Timer errors are unrecoverable.
                        match ready!(Pin::new_unchecked(delay_fut).poll(cx)) {
                            Ok(()) => {}
                            Err(err) => return Poll::Ready(Some(Err(err.into()))),
                        }
                        state.switch_to_request(&stream.api, &stream.options)
                    }
                }
            }
        }
    }
}

impl UpdatesStream {
    /// Creates a new updates stream
    pub fn new(api: Api) -> Self {
        let options = UpdatesStreamOptions::default();
        let state = State::Running(make_request(&api, &options));
        UpdatesStream {
            api,
            options,
            state,
            should_retry: true,
        }
    }

    /// Should retry request when an error has occurred
    ///
    /// Default value is true
    pub fn should_retry(mut self, value: bool) -> Self {
        self.should_retry = value;
        self
    }

    /// Set options
    pub fn options(mut self, options: UpdatesStreamOptions) -> Self {
        self.options = options;
        self
    }
}

impl From<Api> for UpdatesStream {
    fn from(api: Api) -> UpdatesStream {
        UpdatesStream::new(api)
    }
}

/// Update stream options
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UpdatesStreamOptions {
    offset: Integer,
    limit: Integer,
    poll_timeout: Duration,
    error_timeout: Duration,
    allowed_updates: HashSet<AllowedUpdate>,
}

impl UpdatesStreamOptions {
    /// Limits the number of updates to be retrieved
    ///
    /// Values between 1—100 are accepted
    /// Defaults to 100
    pub fn limit(mut self, limit: Integer) -> Self {
        self.limit = limit;
        self
    }

    /// Timeout in seconds for long polling
    ///
    /// 0 - usual short polling
    /// Defaults to 10
    /// Should be positive, short polling should be used for testing purposes only
    pub fn poll_timeout(mut self, poll_timeout: Duration) -> Self {
        self.poll_timeout = poll_timeout;
        self
    }

    /// Timeout in seconds when an error has occurred
    ///
    /// Defaults to 5
    pub fn error_timeout(mut self, error_timeout: u64) -> Self {
        self.error_timeout = Duration::from_secs(error_timeout);
        self
    }

    /// Adds a type of updates you want your bot to receive
    pub fn allowed_update(mut self, allowed_update: AllowedUpdate) -> Self {
        self.allowed_updates.insert(allowed_update);
        self
    }
}

impl Default for UpdatesStreamOptions {
    fn default() -> Self {
        UpdatesStreamOptions {
            offset: 0,
            limit: DEFAULT_LIMIT,
            poll_timeout: DEFAULT_POLL_TIMEOUT,
            error_timeout: DEFAULT_ERROR_TIMEOUT,
            allowed_updates: HashSet::new(),
        }
    }
}
