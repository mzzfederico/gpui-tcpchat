# TCP Chat

A simple Rust workspace containing three crates for TCP-based client-server communication.

## Crates

### 1. `server`
An async TCP chat server that listens on port 8080 and broadcasts messages from any client to all other connected clients. Sends a client_id on first connect.

### 2. `client-lib`
It uses threads and channels to handle bidirectional communication and can receive messages continuously in the background. Sends in messages, receives JSON with metadata.

### 3. `ui`
A basic gpui.rs interface application that uses the `client-lib` to connect to the chat server and participate in group conversations.

## Usage

### Running the Server
```bash
cargo run --bin server
```

The server will start listening on `127.0.0.1:8080`.

### Running the UI Client
In a separate terminal:
```bash
cargo run --bin ui
```

### Building All Crates
```bash
cargo build
```

### Running Tests
```bash
cargo test
```

## Architecture

- **Server**: Uses async tokio with a broadcast message bus for real-time chat
- **Client Library**: Uses channels and threads and defines client logic
- **UI**: Uses GPUI.rs to render the user interface
