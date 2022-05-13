// Copyright 2022 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

/// Implementations of the `rtlambda` API for different HTTP backends.
pub mod backends;
/// A collection of traits and default implementations for them, representing the library's core data structures.
pub mod data;
/// Defines error types and constants.
pub mod error;
/// Defines the [`crate::runtime::LambdaRuntime`] API and provides a default generic implementation.
pub mod runtime;
/// Defines the [`crate::transport::Transport`] abstraction used to support multiple HTTP backends.
pub mod transport;

/// The current Lambda API version used on AWS.
pub static LAMBDA_VER: &str = "2018-06-01";

/// A prelude that contains all the relevant imports when using the library's default runtime implementation,
/// which currently ships with a [ureq](https://crates.io/crates/ureq) based HTTP Backend and [serde_json](https://crates.io/crates/serde_json) for serialization.
pub mod prelude {
    pub use crate::backends::ureq::*;
    pub use crate::data::context::{LambdaContext, RefLambdaContext};
    pub use crate::data::env::LambdaRuntimeEnv;
    pub use crate::runtime::{DefaultRuntime, LambdaRuntime};
    pub use crate::LAMBDA_VER;
}

/// Creates a [`crate::runtime::DefaultRuntime`] with the given response, transport, env, out, err types as well as version and initializer.
#[macro_export]
macro_rules! create_runtime {
    ($response:ty, $transport:ty, $env:ty, $out:ty, $err:ty, $ver:expr, $init:ident) => {
        DefaultRuntime::<$response, $transport, $env, $out, $err>::new($ver, $init);
    };
}

/// Creates a [`crate::runtime::DefaultRuntime`] with ureq based HTTP backend and the default implementation of env-vars handling.
#[macro_export]
macro_rules! default_runtime {
    ($out:ty, $err:ty, $ver:expr, $init:ident) => {
        create_runtime!(
            UreqResponse,
            UreqTransport,
            LambdaRuntimeEnv,
            $out,
            $err,
            $ver,
            $init
        )
    };
}
