# ROCPP - Rust OCPP Implementation

ROCPP is a modular Rust-based implementation of the Open Charge Point Protocol (OCPP), built in Rust.

## Currently Supports

- OCPP 1.6 client-side implementation
- `no_std` support embedded environments
- Desktop charger simulator (Tauri)
- In-house conformance test suite with 60+ test cases

## Planned Features

- OCPP 2.0.x support
- Server-side (Central System) implementation
- Allow external OCPP clients or central systems to connect and run against the conformance test suite via WebSocket, enabling validation of third-party implementations

## Crates

- [`rocpp-core`](./ocpp-core/README.md) — Core protocol types and data structures
- [`rocpp-client`](./ocpp-client/README.md) — Client implementation with conformance tests
- [`rocpp-server`](./ocpp-server/README.md) — Server implementation (WIP)

## Tools & Examples

- [`charger-simulator`](./ocpp-client/examples/v16/simulator/README.md) — OCPP 1.6 charger simulator simulator for testing and debugging

## Contributing

Contributions are welcome!

If you find a bug, want to improve something, or add new test cases, feel free to open an issue or submit a pull request.  
Please keep changes focused and well-scoped. For larger features, open an issue first to discuss the approach.

## License
This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.
