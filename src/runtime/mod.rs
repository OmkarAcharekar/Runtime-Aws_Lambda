// Copyright 2022 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use crate::data::context::RefLambdaContext;
use crate::data::env::RuntimeEnvVars;
use crate::data::response::{LambdaAPIResponse, AWS_FUNC_ERR_TYPE};
use crate::error::{Error, CONTAINER_ERR};
use crate::transport::Transport;

use std::env::set_var;
use std::ffi::OsStr;
use std::fmt::Display;

use serde::Serialize;

// Already handles any panic inducing errors
macro_rules! handle_response {
    ($resp:expr) => {
        let status_code = $resp.get_status_code();
        match status_code {
            400..=499 => {
                let err = $resp.error_response().or(Some("")).unwrap();
                return Err(Error::new(format!(
                    "Client error ({}). ErrorResponse: {}",
                    status_code, err
                )));
            }
            500 => panic!("{}", CONTAINER_ERR),
            _ => (),
        };
    };
}

macro_rules! format_version_string {
    ($version:expr) => {
        if let Some(v) = $version.strip_prefix("/") {
            v.to_string()
        } else {
            $version.to_string()
        }
    };
}

/// A generic trait defining an interface for a Lambda runtime.
/// The HTTP Backend in use is defined by the input types `T` that implements [`Transport`] and `R` implementing [`LambdaAPIResponse`].
/// The `OUT` type parameter is the user-defined response type which represents the success result of the event handler.
///
/// The combination of type parameters enables the compiled program to avoid dynamic dispatch when calling the runtime methods.
pub trait LambdaRuntime<R, T, OUT>
where
    R: LambdaAPIResponse,
    T: Transport<R>,
    OUT: Serialize,
{
    /// Used to fetch the next event from the Lambda service.
    fn next_invocation(&mut self) -> Result<R, Error>;
    /// Sends back a JSON formatted response to the Lambda service, after processing an event.
    fn invocation_response(&self, request_id: &str, response: &OUT) -> Result<R, Error>;
    /// Used to report an error during initialization to the Lambda service.
    fn initialization_error(
        &self,
        error_type: Option<&str>,
        error_req: Option<&str>,
    ) -> Result<R, Error>;
    /// Used to report an error during function invocation to the Lambda service.
    fn invocation_error(
        &self,
        request_id: &str,
        error_type: Option<&str>,
        error_req: Option<&str>,
    ) -> Result<R, Error>;
    /// Implements the runtime loop logic.
    fn run(&mut self);
}

/// The default generic implementation of the [`LambdaRuntime`] interface.
/// Works by accepting a pointer to an initialization function or a closure `initializer` -
/// that is run once and initializes "global" variables that are created once
/// and persist across the runtime's life (DB connections, heap allocated static data etc...).
///
/// The initialization function returns a user-defined closure object that acts as the event handler and can
/// take ownership over those variables by move.
/// The Ok output type of the closure - `OUT` - should implement [`serde::Serialize`].
///
/// The `R`, `T` and `OUT` type parameters correspond to the ones defined in [`LambdaRuntime`].
///
/// The `ENV` type parameter defines the implementation of [`crate::data::env::RuntimeEnvVars`] for reading the env-vars set for the runtime.
///
/// The `ERR` type parameter is a user-defined type representing any error that may occur during initialization or invocation of the event handler.
pub struct DefaultRuntime<R, T, ENV, OUT, ERR>
where
    R: LambdaAPIResponse,
    T: Transport<R>,
    ENV: RuntimeEnvVars,
    //   I: LambdaContext,
    ERR: Display,
    OUT: Serialize,
{
    /// An owned instance of a type implementing [`crate::data::env::RuntimeEnvVars`].
    env_vars: ENV,
    /// The Lambda API version string.
    version: String,
    /// URI of the Lambda API.
    api_base: String,
    /// An owned instance of the HTTP Backend implementing [`crate::transport::Transport`].
    transport: T,
    /// An initialization function that sets up persistent variables and returns the event handler.
    initializer:
        fn()
            -> Result<Box<dyn Fn(Option<&str>, RefLambdaContext<ENV, R>) -> Result<OUT, ERR>>, ERR>,
}

impl<R, T, ENV, OUT, ERR> DefaultRuntime<R, T, ENV, OUT, ERR>
where
    R: LambdaAPIResponse,
    T: Transport<R>,
    ENV: RuntimeEnvVars,
    //   I: LambdaContext,
    ERR: Display,
    OUT: Serialize,
{
    pub fn new(
        version: &str,
        initializer: fn() -> Result<
            Box<dyn Fn(Option<&str>, RefLambdaContext<ENV, R>) -> Result<OUT, ERR>>,
            ERR,
        >,
    ) -> Self {
        // Initialize default env vars and check for the host and port of the runtime API.
        let env_vars = ENV::default();
        let api_base = match env_vars.get_runtime_api() {
            Some(v) => v.to_string(),
            None => panic!("Failed getting API base URL from env vars"),
        };

        // Format the version string, later used in API calls
        let formatted_version: String = format_version_string!(version);

        // Start the transport layer object
        let transport = T::default();

        Self {
            env_vars,
            version: formatted_version,
            api_base,
            transport,
            initializer,
        }
    }

    #[inline(always)]
    pub fn get_env(&self) -> &ENV {
        &self.env_vars
    }
}

impl<R, T, ENV, OUT, ERR> LambdaRuntime<R, T, OUT> for DefaultRuntime<R, T, ENV, OUT, ERR>
where
    R: LambdaAPIResponse,
    T: Transport<R>,
    ENV: RuntimeEnvVars,
    // I: LambdaContext,
    ERR: Display,
    OUT: Serialize,
{
    fn run(&mut self) {
        // Run the app's initializer and check for errors
        let init_result = (self.initializer)();
        let lambda = match init_result {
            Err(init_err) => {
                // Try reporting to the Lambda service if there is an error during initialization
                // TODO: Take error type and request from ERR
                match self.initialization_error(Some("Runtime.InitError"), None) {
                    Ok(r) => r,
                    // If an error occurs during reporting the previous error, panic.
                    Err(err) => panic!(
                        "Failed to report initialization error. Error: {}, AWS Error: {}",
                        &init_err, err
                    ),
                };
                // After reporting an init error just panic.
                panic!("Initialization Error: {}", &init_err);
            }
            // On successfull init, unwrap the underlying closure (event handler)
            Ok(event_handler) => event_handler,
        };

        // Start event processing loop as specified in [https://docs.aws.amazon.com/lambda/latest/dg/runtimes-custom.html]
        loop {
            // Get the next event in the queue.
            // Failing to get the next event will either panic (on server error) or continue (on client-error codes).
            let next: Result<R, _> = self.next_invocation();
            if next.is_err() {
                // TODO - perhaps log the error
                continue;
            }
            let next_resp = next.as_ref().unwrap();
            let request_id = match next_resp.aws_request_id() {
                Some(rid) => rid,
                None => {
                    // TODO - figure out what we'd like to do with the result returned from success/client-err api responses
                    let _ = self.initialization_error(Some("Runtime.MissingRequestId"), None);
                    continue;
                }
            };

            // Create the context object for the lambda execution
            // TODO - Design a way to pass a generic type implementing LambdaContext and use it to construct the context
            let context = RefLambdaContext {
                env_vars: &self.env_vars,
                invo_resp: next_resp,
            };
            // Retrieve the event JSON
            // TODO - deserialize? Currently user code should deserialize inside their handler
            let event = next_resp.event_response();

            // Execute the event handler
            let lambda_output = lambda(event, context);

            // TODO - figure out what we'd like to do with the result returned from success/client-err api responses (e.g: log, run a user defined callback...)
            let _ = match lambda_output {
                Ok(out) => self.invocation_response(request_id, &out),
                // TODO - pass an ErrorRequest json
                Err(err) => {
                    let _err = format!("{}", &err);
                    self.invocation_error(request_id, Some(&_err), Some(&_err))
                }
            };
        }
    }

    fn next_invocation(&mut self) -> Result<R, Error> {
        let url = format!(
            "http://{}/{}/runtime/invocation/next",
            self.api_base, self.version
        );
        let resp = self.transport.get(&url, None, None)?;

        handle_response!(resp);

        // If AWS returns the "Lambda-Runtime-Trace-Id" header, set its value to the -
        // "_X_AMZN_TRACE_ID" env var
        if let Some(req_id) = resp.trace_id() {
            set_var(OsStr::new("_X_AMZN_TRACE_ID"), OsStr::new(req_id));
            self.env_vars.set_trace_id(Some(req_id));
        };

        Ok(resp)
    }

    fn invocation_response(&self, request_id: &str, response: &OUT) -> Result<R, Error> {
        let url = format!(
            "http://{}/{}/runtime/invocation/{}/response",
            self.api_base, self.version, request_id
        );
        // TODO - Utilize a user-defined JSON serializer?
        let serialized = match serde_json::to_string(response) {
            Ok(ser) => ser,
            Err(err) => {
                return Err(Error::new(format!(
                    "Failed serializing output to JSON. {}",
                    err
                )))
            }
        };
        let resp = self.transport.post(&url, Some(&serialized), None)?;

        handle_response!(resp);

        Ok(resp)
    }

    fn initialization_error(
        &self,
        error_type: Option<&str>,
        error_req: Option<&str>,
    ) -> Result<R, Error> {
        let url = format!(
            "http://{}/{}/runtime/init/error",
            self.api_base, self.version
        );
        let headers = error_type.map(|et| (vec![AWS_FUNC_ERR_TYPE], vec![et]));

        let resp = self.transport.post(&url, error_req, headers)?;

        handle_response!(resp);

        Ok(resp)
    }

    fn invocation_error(
        &self,
        request_id: &str,
        error_type: Option<&str>,
        error_req: Option<&str>,
    ) -> Result<R, Error> {
        let url = format!(
            "http://{}/{}/runtime/invocation/{}/error",
            self.api_base, self.version, request_id
        );
        let headers = error_type.map(|et| (vec![AWS_FUNC_ERR_TYPE], vec![et]));

        let resp = self.transport.post(&url, error_req, headers)?;

        handle_response!(resp);

        Ok(resp)
    }
}
