// Copyright 2022 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use crate::data::response::LambdaAPIResponse;
use crate::error::Error;

/// A generic trait that is used as an abstraction to the HTTP client library (AKA "Backend")
/// used to interact with the [runtime API](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html), and the response type returned by that backend.
pub trait Transport<T: LambdaAPIResponse>: Default {
    // TODO - optimize the headers type
    /// Sends an HTTP GET request to the specified `url` with the optional `body` and `headers`.
    fn get(
        &self,
        url: &str,
        body: Option<&str>,
        headers: Option<(Vec<&str>, Vec<&str>)>,
    ) -> Result<T, Error>;
    /// Sends an HTTP POST request to the specified `url` with the optional `body` and `headers`.
    fn post(
        &self,
        url: &str,
        body: Option<&str>,
        headers: Option<(Vec<&str>, Vec<&str>)>,
    ) -> Result<T, Error>;
}
