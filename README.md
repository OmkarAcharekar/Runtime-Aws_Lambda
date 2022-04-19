# Runtime-Aws_Lambda
A Rust runtime for AWS Lambda and a library for creating custom runtimes.


A Rust runtime for [AWS Lambda](https://docs.aws.amazon.com/lambda/latest/dg/welcome.html) and a library for creating custom runtimes.

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

### As a framework
`Runtime-Aws_Lambda`'s API utilizes generic traits - with bounds on their type parameters - to define its interface.

This design minimizes dynamic dispatch calls while allowing the user to:
* Define their own output and error types.
* Choose their HTTP client implementation.
* Implement their own version of internal runtime concerns such as runtime logic, env var handling and context building.

Each trait is provided with a default type implementing it. For example the default HTTP backend is based on [ureq](https://crates.io/crates/ureq). 

The majority of users should be fine with the default implementation and only need to define their output and error types.
Output types should currently implement the [serde::Serialize](https://docs.serde.rs/serde/ser/trait.Serialize.html) trait.
Error types should implement [std::fmt::Display](https://doc.rust-lang.org/std/fmt/trait.Display.html).

## Build and Deploy
`Runtime-Aws_Lambda` is designed to be built into a single executable that contains both your function code and the runtime itself (In AWS terms the runtime "is embedded in the function deployment package").

AWS currently allows you to deploy your function on either `x86` or `aarch64` based machines with either the Amazon Linux 2 OS or the legacy Amazon Linux.

Since Rust is a compiled language, it is therefore required to build your project on the same environment you're planning to deploy to (or to cross-compile it).

Two simple ways to build your code are either using Docker or spinning up an EC2 VM matching your target environment.

### Using Docker
For Amazon Linux 2 you can follow these steps:
 

 1. Clone https://github.com/guyo13/amazon_linux_rust_docker.git
 2. Cd into either the `x86` or `aarch64` dirs and run `docker build .`
 3. Run a container with the built image and mount your crate's root directory to the container's `/opt/src` dir - by adding the `-v /path/to/crate:/opt/src` argument to the `docker run` command.
 4. Connect to the container and execute `cargo build --release`.
 5. **For aarch64 builds** in order to best leverage the AWS Graviton2 CPU, **before** building run `export RUSTFLAGS="-C target-cpu=neoverse-n1 -C target-feature=+lse,+outline-atomics"` **inside** the container.

**This method seems to be viable if your host architecture (eg. x86 laptop/mac) matches your target architecture. Building on an emulated container currently fails on Docker for Mac with rustc 1.60.**  

### Using a VM
- Create a virtual machine on [EC2](https://aws.amazon.com/ec2/getting-started/) that matches your target platform (architecture and OS).
- Install the Rust toolchain using rustup-init ([Instructions](https://rustup.rs/)).
- Clone your crate into the build VM and repeat steps 4-5 above (on the VM. This time there is no container).

### Deploying
After compiling your app on either Docker or a VM, copy the executable binary file into a file named `bootstrap` and zip it into a `function.zip` archive.

Create a Function using the AWS Lambda dashboard or the `aws` cli, make sure that the platform settings (CPU architecture and OS) match your compilation target.
Deploy your `function.zip` archive to the newly created function using any of the [methods supported by AWS](https://docs.aws.amazon.com/lambda/latest/dg/configuration-function-zip.html#configuration-function-update).

