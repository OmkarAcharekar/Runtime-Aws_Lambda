// Copyright 2022 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

/// An enum representing the `InitializationType` choices set as an env-var on the instance by AWS Lambda.
/// See [Defined runtime environment variables](https://docs.aws.amazon.com/lambda/latest/dg/configuration-envvars.html#configuration-envvars-runtime).
#[derive(Clone, Copy, Debug)]
pub enum InitializationType {
    OnDemand,
    ProvisionedConcurrency,
    Unknown,
}

impl InitializationType {
    /// Returns the [`InitializationType`] value corresponding to the input string.
    /// itype must be lowercase.
    fn from_string(itype: &str) -> InitializationType {
        match itype {
            "on-demand" => Self::OnDemand,
            "provisioned-concurrency" => Self::ProvisionedConcurrency,
            // Shouldn't reach here but if for some reason AWS doesn't get it right...
            _ => Self::Unknown,
        }
    }
}

/// An interface trait for reading the environment variables set by the AWS Lambda service.
///
/// Based on - [Defined runtime environment variables](https://docs.aws.amazon.com/lambda/latest/dg/configuration-envvars.html#configuration-envvars-runtime).
pub trait RuntimeEnvVars: Default {
    fn get_handler(&self) -> Option<&str>;
    fn get_region(&self) -> Option<&str>;
    fn get_trace_id(&self) -> Option<&str>;
    fn get_execution_env(&self) -> Option<&str>;
    fn get_function_name(&self) -> Option<&str>;
    fn get_function_memory_size(&self) -> Option<usize>;
    fn get_function_version(&self) -> Option<&str>;
    fn get_initialization_type(&self) -> InitializationType;
    fn get_log_group_name(&self) -> Option<&str>;
    fn get_log_stream_name(&self) -> Option<&str>;
    fn get_access_key(&self) -> Option<&str>;
    fn get_access_key_id(&self) -> Option<&str>;
    fn get_secret_access_key(&self) -> Option<&str>;
    fn get_session_token(&self) -> Option<&str>;
    fn get_runtime_api(&self) -> Option<&str>;
    fn get_task_root(&self) -> Option<&str>;
    fn get_runtime_dir(&self) -> Option<&str>;
    fn get_tz(&self) -> Option<&str>;
    /// Returns the string value of an env-var `var_name` wrapped in an [`Option`],
    /// or `None` if the env-var is not set or the [`std::env::var`] function returns an error.
    fn get_var(var_name: &str) -> Option<String> {
        use std::env;
        env::var(var_name).ok()
    }
    /// Signals that the previous tracing id has changed as a result of a new incoming event.
    fn set_trace_id(&mut self, new_id: Option<&str>);
}

/// A struct implementing [`RuntimeEnvVars`] by caching the default runtime env-vars,
/// and supports a default initialization using std::env::var calls.
#[derive(Debug, Clone)]
pub struct LambdaRuntimeEnv {
    pub handler: Option<String>,
    // This value should be set by the runtime after each next invocation request where a new id is given
    pub trace_id: Option<String>,
    pub region: Option<String>,
    // Custom runtimes currently don't have this value set as per AWS docs
    pub execution_env: Option<String>,
    pub function_name: Option<String>,
    pub function_memory_size: Option<usize>,
    pub function_version: Option<String>,
    pub initialization_type: InitializationType,
    pub log_group_name: Option<String>,
    pub log_stream_name: Option<String>,
    pub access_key: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
    pub runtime_api: Option<String>,
    pub task_root: Option<String>,
    pub runtime_dir: Option<String>,
    pub tz: Option<String>,
}

impl LambdaRuntimeEnv {
    /// Constructs a new [`LambdaRuntimeEnv`] by reading the process' environment variables,
    /// and caching the [default env-vars](https://docs.aws.amazon.com/lambda/latest/dg/configuration-envvars.html#configuration-envvars-runtime).
    pub fn from_env() -> LambdaRuntimeEnv {
        use std::env;
        LambdaRuntimeEnv {
            handler: env::var("_HANDLER").ok(),
            region: env::var("AWS_REGION").ok(),
            trace_id: None,
            execution_env: env::var("AWS_EXECUTION_ENV").ok(),
            function_name: env::var("AWS_LAMBDA_FUNCTION_NAME").ok(),
            function_memory_size: match env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE").ok() {
                Some(v) => v.parse::<usize>().ok(),
                None => None,
            },
            function_version: env::var("AWS_LAMBDA_FUNCTION_VERSION").ok(),
            initialization_type: match env::var("AWS_LAMBDA_INITIALIZATION_TYPE").ok() {
                Some(v) => InitializationType::from_string(&v),
                None => InitializationType::Unknown,
            },
            log_group_name: env::var("AWS_LAMBDA_LOG_GROUP_NAME").ok(),
            log_stream_name: env::var("AWS_LAMBDA_LOG_STREAM_NAME").ok(),
            access_key: env::var("AWS_ACCESS_KEY").ok(),
            access_key_id: env::var("AWS_ACCESS_KEY_ID").ok(),
            secret_access_key: env::var("AWS_SECRET_ACCESS_KEY").ok(),
            session_token: env::var("AWS_SESSION_TOKEN").ok(),
            runtime_api: env::var("AWS_LAMBDA_RUNTIME_API").ok(),
            task_root: env::var("LAMBDA_TASK_ROOT").ok(),
            runtime_dir: env::var("LAMBDA_RUNTIME_DIR").ok(),
            tz: env::var("TZ").ok(),
        }
    }
}

impl Default for LambdaRuntimeEnv {
    fn default() -> Self {
        Self::from_env()
    }
}

impl RuntimeEnvVars for LambdaRuntimeEnv {
    #[inline(always)]
    fn get_handler(&self) -> Option<&str> {
        self.handler.as_deref()
    }

    #[inline(always)]
    fn get_region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    #[inline(always)]
    fn get_trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    #[inline(always)]
    fn get_execution_env(&self) -> Option<&str> {
        self.execution_env.as_deref()
    }

    #[inline(always)]
    fn get_function_name(&self) -> Option<&str> {
        self.function_name.as_deref()
    }

    #[inline(always)]
    fn get_function_memory_size(&self) -> Option<usize> {
        self.function_memory_size
    }

    #[inline(always)]
    fn get_function_version(&self) -> Option<&str> {
        self.function_version.as_deref()
    }

    #[inline(always)]
    fn get_initialization_type(&self) -> InitializationType {
        self.initialization_type
    }
    #[inline(always)]
    fn get_log_group_name(&self) -> Option<&str> {
        self.log_group_name.as_deref()
    }

    #[inline(always)]
    fn get_log_stream_name(&self) -> Option<&str> {
        self.log_stream_name.as_deref()
    }

    #[inline(always)]
    fn get_access_key(&self) -> Option<&str> {
        self.access_key.as_deref()
    }

    #[inline(always)]
    fn get_access_key_id(&self) -> Option<&str> {
        self.access_key_id.as_deref()
    }

    #[inline(always)]
    fn get_secret_access_key(&self) -> Option<&str> {
        self.secret_access_key.as_deref()
    }

    #[inline(always)]
    fn get_session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
    }

    #[inline(always)]
    fn get_runtime_api(&self) -> Option<&str> {
        self.runtime_api.as_deref()
    }

    #[inline(always)]
    fn get_task_root(&self) -> Option<&str> {
        self.task_root.as_deref()
    }

    #[inline(always)]
    fn get_runtime_dir(&self) -> Option<&str> {
        self.runtime_dir.as_deref()
    }

    #[inline(always)]
    fn get_tz(&self) -> Option<&str> {
        self.tz.as_deref()
    }

    #[inline]
    fn set_trace_id(&mut self, new_id: Option<&str>) {
        self.trace_id = new_id.map(|v| v.to_string());
    }
}
