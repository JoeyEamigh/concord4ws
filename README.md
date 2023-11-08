# Concord4 WebSockets

concord4ws is a websocket server for interacting with the Concord4 alarm system. It is designed to be run in a container that has access to the serial port connected to the Interlogix Superbus 2000. It is written in Rust and uses my [concord4-rs](https://github.com/JoeyEamigh/concord4-rs) library to communicate with the alarm panel. It was created to be used with Home Assistant via [concord4ws-ha](https://github.com/JoeyEamigh/concord4ws-ha).

## Running

```bash
docker run --name concord4ws \
  --device=/dev/ttyUSB0 \
  -p 8080:8080 \
  -e SERIAL_DEVICE=/dev/ttyUSB0 \
  -e SOCKET_PORT=8080 \
  ghcr.io/joeyeamigh/concord4ws:latest
```

## Configuration

| Environment Variable | Description | Default |
| -------------------- | ----------- | ------- |
| SERIAL_DEVICE | The serial device to use to communicate with the alarm panel. | (panic) |
| SOCKET_PORT | The port to listen on for websocket connections. | 8080 |

## Setup with Home Assistant

1. Setup the websocket server as described above.
2. Setup the Home Assistant component as described in [concord4ws-ha](https://github.com/JoeyEamigh/concord4ws-ha).
3. Restart Home Assistant.
4. On the integrations page, search for "Concord WebSocket", then enter the websocket server information.

## Development

This project is four projects in one. The [first submodule](./concord4-rs) is the library to communicate with the panel, and can be found at [concord4-rs](https://github.com/JoeyEamigh/concord4-rs). The [second](./src) is this project itself, which is the websocket server written in Rust that consumes that library. The [third submodule](./concord4ws-py) is a python library that can be used to interact with the websocket server, and can be found at [concord4ws-py](https://github.com/JoeyEamigh/concord4ws-py). Finally, the Home Assistant integration is in the [fourth submodule](./concord4ws-ha), and can be found at [concord4ws-ha](https://github.com/JoeyEamigh/concord4ws-ha).

```bash
git clone --recursive https://github.com/JoeyEamigh/concord4ws.git
cd concord4ws
rustup default stable
cargo run -F dotenv # this will load the .env file
```

## Notes

This project is still under construction, and more features will be added as I have time.
