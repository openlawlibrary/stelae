//! Tracing/logging for HTTP servers

use std::time::Instant;

use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    HttpMessage,
};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder};

/// The length of time in milliseconds after which a request is considered slow
const SLOW_REQUEST_MS: u128 = 5 * 1000;

/// More or less an alias just to add custom functionality to `DefaultRootSpanBuilder`
pub struct StelaeRootSpanBuilder;

/// For measuring the duration of a request
struct RequestStart(Instant);

impl RootSpanBuilder for StelaeRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> tracing::Span {
        // The `{}` block tells the compiler to return ownership of `request`.
        // NOTE:
        //   Because the `request` variable is included in the `*span!` macro, we're not likely to
        //   get linting feedback that the macro also mutably borrows `request`. Or at least I
        //   think that's why I only discovered the second mutable borrow _after_ running the app.
        //   Quite unusual for Rust to not pick up on it at compile time.
        {
            let mut request_extensions = request.extensions_mut();
            request_extensions.insert(RequestStart(Instant::now()));
        }

        // The `RootSpan` is the data that is included with every `tracing::*` call during the
        // lifetime of a HTTP request. It contains things like the user agent, HTTP path, etc.
        // It can get quite noisy when tracing lots of non HTTP-related activity. But that is
        // likely the fair price to pay for being able to sanely associate the log line with
        // the request. Recall that in production there are likely to be many simultaneous requests
        // making it hard to "read" the journey of a single request. A unique `request_id` is also
        // included, so it would certainly be possible to disable the verbose default data, and
        // then manually match the HTTP request's `request_id` with other log lines' `request_id`s.
        tracing_actix_web::root_span!(
            request,
            duration_ms = tracing::field::Empty,
            duration_ns = tracing::field::Empty,
        )
    }

    fn on_request_end<B>(
        span: tracing::Span,
        outcome: &Result<ServiceResponse<B>, actix_web::Error>,
    ) {
        // TODO:
        //   I couldn't find a way of triggering the case where `outcome` is
        //   `Result::Err(actix_web::Error)`. It doesn't seem to be when a route method returns an
        //   error, as I assume that's considered a handled error. So maybe `outcome` is only ever
        //   an error for an Actix-internal error? Either way, the root span and timings all work
        //   normally for known and handled request errors.
        let () = outcome.as_ref().map_or((), |response| {
            if let Some(req_start) = response.request().extensions().get::<RequestStart>() {
                let elapsed = req_start.0.elapsed();
                let millis = elapsed.as_millis();
                // Add the timings to the default `RootSpan`
                span.record("duration_ms", millis);
                span.record("duration_ns", elapsed.as_nanos());
                if millis > SLOW_REQUEST_MS {
                    tracing::warn!(duration_ms = millis, "Slow HTTP request");
                } else {
                    tracing::trace!("HTTP Request");
                }
            }
        });
        // Captures the standard `RootSpan` fields
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
