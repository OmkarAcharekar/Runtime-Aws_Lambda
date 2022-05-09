// Copyright 2022 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use std::time::Duration;

pub static AWS_REQ_ID: &str = "Lambda-Runtime-Aws-Request-Id";
pub static AWS_DEADLINE_MS: &str = "Lambda-Runtime-Deadline-Ms";
pub static AWS_FUNC_ARN: &str = "Lambda-Runtime-Invoked-Function-Arn";
pub static AWS_TRACE_ID: &str = "Lambda-Runtime-Trace-Id";
pub static AWS_CLIENT_CTX: &str = "Lambda-Runtime-Client-Context";
pub static AWS_COG_ID: &str = "Lambda-Runtime-Cognito-Identity";
pub static AWS_FUNC_ERR_TYPE: &str = "Lambda-Runtime-Function-Error-Type";

//Based on [https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html#runtimes-api-next]
/// An interface trait representing a response from the [AWS Lambda runtime API](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).
///
/// It encapsulates all of the 4 possible response types defined by the runtime API.
/// Differentiating between them and correctly reading the response are implementation details.
///
/// Implementations of this trait *should* enable reading data without requiring ownership of or exclusive reference to the type,
/// therefore it is **not** always possible to implement it **directly** on HTTP Response types exposed by different vendors -
/// for example reading the body from a [ureq::Response](https://docs.rs/ureq/2.4.0/ureq/struct.Response.html#method.into_string) instance
/// moves the instance, making reading any header (for example request id) or the response's status code impossible -
/// or failing entirely if a reference to the instance already exist such as when using a [`crate::data::context::RefLambdaContext`] context implementation.
///
/// A good approach is to implement this trait on a wrapper type that caches the relevant headers and reads the body,
/// for an example see [`crate::backends::ureq::UreqResponse`].
pub trait LambdaAPIResponse {
    fn get_body(&self) -> Option<&str>;
    fn get_status_code(&self) -> u16;
    fn aws_request_id(&self) -> Option<&str>;
    fn deadline(&self) -> Option<Duration>;
    fn invoked_function_arn(&self) -> Option<&str>;
    fn trace_id(&self) -> Option<&str>;
    fn client_context(&self) -> Option<&str>;
    fn cognito_identity(&self) -> Option<&str>;

    // TODO: find out whether lambda might send a non-UTF-8 encoded json event and change signature if needed
    fn event_response(&self) -> Option<&str> {
        match self.is_success() {
            true => self.get_body(),
            false => None,
        }
    }

    fn error_response(&self) -> Option<&str> {
        match self.is_client_err() {
            true => self.get_body(),
            false => None,
        }
    }

    #[inline]
    fn status_response(&self) -> Option<&str> {
        // TODO - only return if the response type is defined to return a StatusResponse object
        self.event_response()
    }

    fn is_success(&self) -> bool {
        matches!(self.get_status_code(), 200..=299)
    }

    fn is_client_err(&self) -> bool {
        matches!(self.get_status_code(), 400..=499)
    }

    fn is_server_err(&self) -> bool {
        matches!(self.get_status_code(), 500..=599)
    }

    fn is_err(&self) -> bool {
        matches!(self.get_status_code(), 400..=499 | 500..=599)
    }
}
