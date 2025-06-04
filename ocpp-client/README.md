# `rocpp-client`

OCPP 1.6 client-side implementation for EV chargers.

### Features

- Full charge point asynchronous state machine
- Boot, heartbeat, transaction, config, firmware, diagnostics, reservation, and reset support
- Tauri-based desktop simulator
- `no_std + alloc` support for embedded targets
- 60+ conformance tests

### Run the Simulator
See the Simulator [README](./examples/v16/simulator/README.md) for details.
```sh
cd ocpp-client/examples/v16/simulator
npm run tauri dev
```

## Conformance Testing

Includes an in-house conformance test suite located in [ocpp-client/tests/conformance](./tests/).  
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