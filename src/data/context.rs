// Copyright 2022 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use crate::data::env::RuntimeEnvVars;
use crate::data::response::LambdaAPIResponse;
use crate::error::Error;
use std::time::Duration;

/// An interface trait that should be implemented by types representing a [Context object]([https://docs.aws.amazon.com/lambda/latest/dg/python-context.html]).
///
/// The context object exposes constant data from the instance's environment variables,
/// as well as data - such as request id and execution deadline - that is specific to each event.
pub trait LambdaContext {
    /// A default implementation that calculates the time difference between its time of invocation and the
    /// handler execution deadline specified by AWS Lambda.
    fn get_remaining_time_ms(&self) -> Result<Duration, Error> {
        let now = std::time::SystemTime::now();
        match now.duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(now_since_epoch) => match self.get_deadline() {
                Some(dur) => dur,
                None => return Err(Error::new("Missing deadline info".to_string())),
            }
            .checked_sub(now_since_epoch)
            .ok_or_else(|| Error::new("Duration error".to_string())),
            Err(e) => Err(Error::new(e.to_string())),
        }
    }
    // Per-invocation data (event-related)
    fn get_deadline(&self) -> Option<Duration>;
    fn invoked_function_arn(&self) -> Option<&str>;
    fn aws_request_id(&self) -> Option<&str>;
    // Per-runtime data (constant accross the lifetime of the runtime, taken from env-vars)
    fn function_name(&self) -> Option<&str>;
    fn function_version(&self) -> Option<&str>;
    fn memory_limit_in_mb(&self) -> Option<usize>;
    fn log_group_name(&self) -> Option<&str>;
    fn log_stream_name(&self) -> Option<&str>;
    // Identity and Client context - see [https://docs.aws.amazon.com/lambda/latest/dg/python-context.html]
    // TODO - parse these structures and return a relevant type
    fn cognito_identity(&self) -> Option<&str>;
    fn client_context(&self) -> Option<&str>;
}

/// A generic implementation of [`LambdaContext`] that relies on **borrowing** existing owned
/// instances of types that implement [`crate::data::env::RuntimeEnvVars`] - for reading environment variables -
/// and [`crate::data::response::LambdaAPIResponse`] - for reading event-related data.
///
/// It can be used so long that its lifetime is less than or equal to its referents.
///
/// This implementation is used to avoid needlessly copying data that is immutable by definition,
/// however it is assumed that types implementing [`crate::data::response::LambdaAPIResponse`] can be read from
/// immutably - which is not the always case with HTTP Response types,
/// for example [ureq::Response](https://docs.rs/ureq/2.4.0/ureq/struct.Response.html#method.into_string) consumes itself upon reading the response body.
/// See [`crate::data::response::LambdaAPIResponse`].
pub struct RefLambdaContext<'a, E, R>
where
    E: RuntimeEnvVars,
    R: LambdaAPIResponse,
{
    /// A shared reference to a type implementing [`crate::data::env::RuntimeEnvVars`].
    pub env_vars: &'a E,
    /// A shared reference to a type implementing [`crate::data::response::LambdaAPIResponse`].
    pub invo_resp: &'a R,
}

impl<'a, E, R> LambdaContext for RefLambdaContext<'a, E, R>
where
    E: RuntimeEnvVars,
    R: LambdaAPIResponse,
{
    #[inline]
    fn get_deadline(&self) -> Option<Duration> {
        self.invo_resp.deadline()
    }

    #[inline(always)]
    fn invoked_function_arn(&self) -> Option<&str> {
        self.invo_resp.invoked_function_arn()
    }

    #[inline(always)]
    fn aws_request_id(&self) -> Option<&str> {
        self.invo_resp.aws_request_id()
    }

    #[inline(always)]
    fn function_name(&self) -> Option<&str> {
        self.env_vars.get_function_name()
    }

    #[inline(always)]
    fn function_version(&self) -> Option<&str> {
        self.env_vars.get_function_version()
    }

    #[inline(always)]
    fn memory_limit_in_mb(&self) -> Option<usize> {
        self.env_vars.get_function_memory_size()
    }

    #[inline(always)]
    fn log_group_name(&self) -> Option<&str> {
        self.env_vars.get_log_group_name()
    }

    #[inline(always)]
    fn log_stream_name(&self) -> Option<&str> {
        self.env_vars.get_log_stream_name()
    }

    #[inline(always)]
    fn cognito_identity(&self) -> Option<&str> {
        self.invo_resp.cognito_identity()
    }

    #[inline(always)]
    fn client_context(&self) -> Option<&str> {
        self.invo_resp.client_context()
    }
}
