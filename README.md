

![introduce](./docs/images/servicefilter.svg "")

# What is Servicefilter
The goal of Servicefilter is to make business programs more focused on business logic.

Thinking from the perspective of data flow, the business application obtains the request data externally, processes it based on the request data and combines it with the data obtained from the environment (including obtaining it from the database or other services), and returns the processed data to the requester. In this process, data processing is the business. Receiving requests and obtaining data from the environment are auxiliary to the business. Simplifying request processing and obtaining data processing can allow the program to focus more on the business. Using a unified or single protocol to allow applications to provide external services and obtain data can effectively simplify applications. For example, using a unified protocol in a microservice environment can effectively reduce the caller's dependence and configuration. Consistent with using a unified protocol for calls between services, middleware provides the same protocol to services to further reduce service dependencies and configurations.

In order to implement applications using a single protocol for services, create a Servicefilter application to proxy all requests of the application. Other advantages that this approach can bring include:

1 Reduced dependence. If the business uses the same protocol as the call between services, there is no need to refer to additional dependencies. At the same time, it will bring about fewer conflicts, fewer bugs in dependencies, smaller package size, faster startup, and faster release.

2 Reduce the configuration in the program. Middleware configurations such as passwords and number of connections are managed uniformly by the configuration system. 

3 Operation and maintenance and business are more independent. The dynamic configuration and capability enhancement of middleware can be handled independently of the business itself, making it easier to manage and upgrade.

4 Easy to debug. The data all passes through the middle layer, which can facilitate tests such as network disconnection, slow network, and data simulation.

# Getting started
## Concept
Applications are designed to serve multiple business services simultaneously. Business services can be provided in the form of network services or plug-ins. The application provides 6 types of filters to handle requests. The application can load a crate that implements the filter trait to handle the request (do not panic in the crate, otherwise it will cause the entire application to stop https://rust-lang.zulipchat.com/#narrow/stream/210922-project-ffi-unwind) and https://github.com/rust-lang/rust/issues/83994.

## Running
The sample configuration file required for application startup is placed in ${workspaceFolder}/servicefilter-main/src/service-filter-default.yaml .
Start app cmd: cargo run -- -c servicefilter-main/src/service-filter-default.yaml
If you use vscod to debug the application, .vscode/launch.json already configured

# Notable crates in this workspace
* servicefilter-core: Core trait definition
* servicefilter-filter: Common filter provided by servicefilter
* servicefilter-lib: Crate of common middleware connections provided by servicefilter
* servicefilter-load: Load crates provided by servicefilter or third parties, the trait is defined in servicefilter-core
* servicefilter-main: Servicefilter application starts crate
* servicefilter-protocol: External and internal network protocols provided by proxy services

# TODO
* Dynamic configuration
* Add wasm support
* Add metrics - prometheus
* Add graceful upgrade
    * filter change
    * restart without close client connection
        * systemfd — a systemd compatible socket server that passes sockets to a subprocess
        * listenfd — a crate to consume externally passed sockets

# Dependencies

In order to build, you need the `protoc` Protocol Buffers compiler, along with Protocol Buffers resource files.

#### Ubuntu

```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y protobuf-compiler libprotobuf-dev
```

#### Alpine Linux

```sh
sudo apk add protoc protobuf-dev
```

#### macOS

Assuming [Homebrew](https://brew.sh/) is already installed. (If not, see instructions for installing Homebrew on [the Homebrew website](https://brew.sh/).)

```zsh
brew install protobuf
```

#### Windows

- Download the latest version of `protoc-xx.y-win64.zip` from [HERE](https://github.com/protocolbuffers/protobuf/releases/latest)
- Extract the file `bin\protoc.exe` and put it somewhere in the `PATH`
- Verify installation by opening a command prompt and enter `protoc --version`
