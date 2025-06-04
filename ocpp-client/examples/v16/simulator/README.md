# OCPP 1.6 Desktop Simulator

A Tauri-based desktop application to simulate an OCPP 1.6 EV charge point using the ROCPP client library.

Useful for testing backend systems, debugging protocol behavior, and visualizing state transitions in a GUI.

## Features

- Connects to any OCPP 1.6 Central System via WebSocket
- Simulates boot, heartbeats, transactions, and more
- Interactive UI for sending remote commands and observing responses
- Real-time log streaming and debug output

## Run the Simulator

```sh
# From the root of the repo:
cd ocpp-client/examples/v16/simulator
npm run tauri dev