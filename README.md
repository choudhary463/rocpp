# ROCPP - Rust OCPP Implementation

A Rust-based implementation of the Open Charge Point Protocol (OCPP) 1.6 for electric vehicle charging infrastructure. The project currently includes OCPP **client** implementation  with simulator, with plans to support **server-side** functionality, **no-std embedded environments** and **OCPP2.x** support in the future.

## Project Structure

The project is organized into several crates:

* `ocpp-core`: Core protocol definitions, message formats, and data types
* `ocpp-client`: Client implementation with state machine and services
* `ocpp-server`: Server-side implementation (WIP)

## Getting Started (Client)

### Build
Clone the repo and build the project:

```sh
git clone https://github.com/choudhary463/rocpp.git
cd rocpp
cargo build --workspace
```

### Run the Simulator
To launch the desktop simulator:
```sh
cd ocpp-client/examples/v16/simulator
cargo tauri dev
```

## Conformance Testing

ROCPP includes an in-house conformance test suite located in [ocpp-client/tests/conformance](https://github.com/choudhary463/rocpp/blob/main/ocpp-client/tests/conformance).  
Currently, **60+ test cases** have been implemented to verify behavior against the OCPP 1.6 specification, covering:

- Boot and heartbeat
- Authentication and authorization
- Transaction lifecycle (start/stop)
- Configuration key handling
- Firmware update and diagnostics
- Reservation management
- Remote trigger and reset
- Error handling and edge cases

You can run all tests using:

```sh
cd ocpp-client
cargo test --test conformance_runner
```

### External Compatibility (Planned)
Future versions will allow external OCPP clients or central systems to connect and run against this test suite via WebSocket.
This will enable developers to validate their own implementations against the ROCPP test harness.

## Contributing

Contributions are welcome!

If you find a bug, want to improve something, or add new test cases, feel free to open an issue or submit a pull request.  
Please keep changes focused and well-scoped. For larger features, open an issue first to discuss the approach.

## License
This project is licensed under the MIT License - see the [LICENSE](https://github.com/choudhary463/rocpp/blob/main/LICENSE) file for details.
