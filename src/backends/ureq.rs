// Copyright 2022 Guy Or and the "Runtime-Aws_Lambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use crate::data::response::*;
use crate::error::Error;
use crate::transport::Transport;
use ureq::Agent;
use ureq::Response;

use std::time::Duration;

macro_rules! copy_str_header {
    ($resp:expr, $header:expr) => {
        $resp.header($header).map(|v| v.to_string())
    };
}

/// A wrapper that processes a [ureq::Response] and implements the [`crate::data::response::LambdaAPIResponse`] trait.
pub struct UreqResponse {
    body: Option<String>,
    status: u16,
    _request_id: String,
    _deadline: Option<Duration>,
    _arn: Option<String>,
    _trace_id: Option<String>,
    _cognito_id: Option<String>,
    _client_context: Option<String>,
}

impl UreqResponse {
    /// A constructor that consumes a [ureq::Response] by copying the relevant headers and reading the request body.
    fn from_response(resp: Response) -> Result<Self, Error> {
        // Copy status
        let status = resp.status();

        // Copy AWS headers
        let _request_id = match resp.header(AWS_REQ_ID) {
            Some(v) => v.to_string(),
            None => {
                return Err(Error::new(
                    "Missing Lambda-Runtime-Aws-Request-Id header".to_string(),
                ))
            }
        };

        // Parse milliseconds to Duration
        let _deadline = match resp.header(AWS_DEADLINE_MS) {
            Some(ms) => match ms.parse::<u64>() {
                Ok(val) => Some(Duration::from_millis(val)),
                Err(_) => None,
            },
            None => None,
        };
        let _arn = copy_str_header!(resp, AWS_FUNC_ARN);
        let _trace_id = copy_str_header!(resp, AWS_TRACE_ID);
        let _cognito_id = copy_str_header!(resp, AWS_COG_ID);
        let _client_context = copy_str_header!(resp, AWS_CLIENT_CTX);

        // Consume the response into a string
        let body = match resp.into_string() {
            Ok(data) => Some(data),
            Err(err) => return Err(Error::new(format!("{}", err))),
        };

        Ok(Self {
            body,
            status,
            _request_id,
            _deadline,
            _arn,
            _trace_id,
            _cognito_id,
            _client_context,
        })
    }
}

impl LambdaAPIResponse for UreqResponse {
    #[inline(always)]
    fn get_body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    #[inline(always)]
    fn get_status_code(&self) -> u16 {
        self.status
    }

    #[inline]
    fn aws_request_id(&self) -> Option<&str> {
        Some(&self._request_id)
    }
    #[inline]
    fn deadline(&self) -> Option<Duration> {
        self._deadline
    }
    #[inline]
    fn invoked_function_arn(&self) -> Option<&str> {
        self._arn.as_deref()
    }
    #[inline]
    fn trace_id(&self) -> Option<&str> {
        self._trace_id.as_deref()
    }
    #[inline]
    fn client_context(&self) -> Option<&str> {
        self._client_context.as_deref()
    }
    #[inline]
    fn cognito_identity(&self) -> Option<&str> {
        self._cognito_id.as_deref()
    }
}

/// Wraps a [`ureq::Agent`] to implement the [`crate::transport::Transport`] trait.
/// Contains a specialized implementation for [`UreqResponse`] type parameter.
///
/// AWS runtime instructs the implementation to disable timeout on the next invocation call.
/// This implementation achieves this by creating a [`ureq::Agent`] with 1 day in seconds of timeout.
pub struct UreqTransport {
    agent: Agent,
}

impl UreqTransport {
    /// Creates a new transport objects with an underlying [ureq::Agent] that will (practically) not time out.
    fn new() -> Self {
        let agent = ureq::builder().timeout(Duration::from_secs(86400)).build();
        UreqTransport { agent }
    }

    /// Sends a request using the underlying agent.
    fn request(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
        headers: Option<(Vec<&str>, Vec<&str>)>,
    ) -> Result<Response, Error> {
        let mut req = self.agent.request(method, url);
        if let Some(headers) = headers {
            let (keys, values) = headers;
            let len = std::cmp::min(keys.len(), values.len());
            for i in 0..len {
                req = req.set(keys[i], values[i]);
            }
        }
        if let Some(body) = body {
            return req
                .send_string(body)
                .map_err(|err| Error::new(format!("{}", err)));
        }
        req.call().map_err(|err| Error::new(format!("{}", err)))
    }
}

impl Default for UreqTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport<UreqResponse> for UreqTransport {
    fn get(
        &self,
        url: &str,
        body: Option<&str>,
        headers: Option<(Vec<&str>, Vec<&str>)>,
    ) -> Result<UreqResponse, Error> {
        let res = self.request("GET", url, body, headers);
        if let Ok(res) = res {
            return UreqResponse::from_response(res);
        }
        Err(res.unwrap_err())
    }

    fn post(
        &self,
        url: &str,
        body: Option<&str>,
        headers: Option<(Vec<&str>, Vec<&str>)>,
    ) -> Result<UreqResponse, Error> {
        let res = self.request("POST", url, body, headers);
        if let Ok(res) = res {
            return UreqResponse::from_response(res);
        }
        Err(res.unwrap_err())
    }
}
