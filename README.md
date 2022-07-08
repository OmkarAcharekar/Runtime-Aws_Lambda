# Runtime-Aws_Lambda



Runtime for [Amazon Web Services](https://docs.aws.amazon.com/lambda/latest/dg/welcome.html) also library for creating runtimes.

The default runtime implementation shipped  lets you write native Rust event handlers for AWS Lambda.
 
Your handler code gets compiled along with the runtime into a single executable file which is deployed to the Lambda service.

It keeps dependencies and complexity to a minimum and does not depend on `tokio`. Writing `Runtime-Aws_Lambda` functions is simple and easy.

## Usage
To get started, you may adapt the echo-server example:

Add `Runtime-Aws_Lambda` and `serde` as a dependency to your `Cargo.toml` file:

```toml
[dependencies]
Runtime-Aws_Lambda = "0.0.1"
serde = { version = "1", features = ["derive"] }
```

And in your `main.rs` file:
```rust
use Runtime-Aws_Lambda::prelude::*;
use serde::Serialize;

// Import the [`default_runtime`] macro from Runtime-Aws_Lambda.
#[macro_use] extern crate Runtime-Aws_Lambda;

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
    // Your one-time initialization logic goes here:

    //

    // Return the event handler closure
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

        // Runtime-Aws_Lambda leaves use-case specific concerns such as event JSON deserialization to the handler.
        // In this example we do not deserialize the event. Use serde_json or any other library to perform deserialization if needed.

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
```

## How does it work
Your main function code creates and runs the runtime, which invokes your handler on incoming events and handles errors if any. 
 
 A typical setup consists of:
* Creating a new binary crate and including `Runtime-Aws_Lambda` as a dependency in your `Cargo.toml` file.
* Importing the prelude - `Runtime-Aws_Lambda::prelude::*` in the `main.rs` file.
* Writing an initialization function that contains one-time initialization code and returns a closure - containing the event handling logic (the business logic of your lambda).
* In your main function, creating a new `DefaultRuntime` passing the Lambda API version and a pointer to your initialization function.
* Calling the `run()` method on the runtime instance to start the runtime.

