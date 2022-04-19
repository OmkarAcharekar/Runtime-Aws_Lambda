use Runtime-Aws_Lambda::prelude::*;
use serde::Serialize;

// Import the [`default_runtime`] macro from Runtime-Aws_Lambda.
#[macro_use]
extern crate Runtime-Aws_Lambda;

// Create a struct representing the lambda's response, and derive the [`serde::Serialize`] trait.
#[derive(Serialize, Clone)]
struct EchoMessage {
    msg: String,
    req_id: String,
}

// Define output and error types for berevity.
// The Output type must implement [`serde::Serialize`]
type OUT = EchoMessage;
// The error type must implement the `Display` trait
type ERR = String;

// Implement an initialization function.
// The initialization function returns a Result with the Ok type resolving to a dynamically allocated
// closure that accepts the Event from Lambda (as an optional string) and the context object.
// The closure itself returns a Result with the Ok and Err types being the previously defined `OUT` and `ERR` types respectively.
// The initialization function may fail (e.g if a db connection was not succesfully opened, etc..) and in that case
// the function should return an Err variant of the same `ERR` type defined for the event handler.
fn initialize() -> Result<
    Box<dyn Fn(Option<&str>, RefLambdaContext<LambdaRuntimeEnv, UreqResponse>) -> Result<OUT, ERR>>,
    ERR,
> {
    return Ok(Box::new(move |event, context| {
        // Get the aws request id
        let req_id = context.aws_request_id().unwrap();

        // Unwrap the event string
        let event = match event {
            Some(v) => v,
            None => {
                return Err(format!(
                    "AWS should not permit empty events. Something strange must've happened."
                ))
            }
        };

        if event == "\"\"" {
            return Err(format!("Empty input, nothing to echo."));
        }

        // Echo the event back as a string.
        Ok(EchoMessage {
            msg: format!("ECHO: {}", event),
            req_id: req_id.to_string(),
        })
    }));
}

fn main() {
    // Create a runtime instance and run its loop.
    // This is the equivalent of:
    // let mut runtime =  DefaultRuntime::<UreqResponse, UreqTransport, LambdaRuntimeEnv, OUT, ERR>::new(LAMBDA_VER, initialize);
    let mut runtime = default_runtime!(OUT, ERR, LAMBDA_VER, initialize);

    runtime.run();
}
