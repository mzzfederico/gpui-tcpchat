# Arox Rust Workspace

A simple Rust workspace containing three crates for TCP-based client-server communication.

## Crates

### 1. `server`
An async TCP chat server that listens on port 8080 and broadcasts messages from any client to all other connected clients.

### 2. `client-lib`
A library crate that provides a simple client interface for connecting to the chat server. It uses threads and channels to handle bidirectional communication and can receive messages continuously in the background.

### 3. `ui`
A command-line interface application that uses the `client-lib` to connect to the chat server and participate in group conversations.

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

This will connect to the chat server and provide an interactive prompt. Type messages and press Enter to send them to all other connected clients. Messages from other clients will appear automatically. Type `quit` to exit.

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
- **Client Library**: Uses channels and threads for non-blocking bidirectional communication
- **UI**: Simple stdin/stdout interface with background message listening

All crates use `anyhow` for error handling to keep things simple and ergonomic.

## Example Chat Session

**Terminal 1 (Client 1):**
```
$ cargo run --bin ui
Connecting to server...
Connected! Type messages and press Enter to send them.
Type 'quit' to exit.

> Hello everyone!
> How is everyone doing?
< Client 2: Hi there!
< Client 3: Good morning!
> quit
Goodbye!
```

**Terminal 2 (Client 2):**
```
$ cargo run --bin ui
Connecting to server...
Connected! Type messages and press Enter to send them.
Type 'quit' to exit.

> Hi there!
< Client 1: Hello everyone!
< Client 1: How is everyone doing?
< Client 3: Good morning!
```
