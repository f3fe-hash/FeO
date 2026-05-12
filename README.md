<!-- Badges --> <p align="center"> <img src="https://img.shields.io/github/stars/f3fe-hash/FeO.svg" /> <img src="https://img.shields.io/github/forks/f3fe-hash/FeO.svg" /> <img src="https://img.shields.io/github/tag/f3fe-hash/FeO.svg" /> <img src="https://img.shields.io/github/release/f3fe-hash/FeO.svg" /> <img src="https://img.shields.io/github/issues/f3fe-hash/FeO.svg" /> <img src="https://img.shields.io/bower/v/FeO.svg" /> </p>
FeO
===

FeO is a Linux-oriented runtime and tooling set for developing and running small "nodes" (programs) used in robotics and embedded workflows. It combines C for low-level runtime components and Rust for networking/management tools.

**Key Features**
- **TLS transport:** `fup` (client) connects to the runtime using TLS.
- **Framed protocol:** All packets use a simple header framing: a `Content-Length: <N>` header followed by a CRLFCRLF (`\r\n\r\n`) separator and then an N-byte binary body.
- **Client tools (`fup`)**: supports `upload` and `start` subcommands to upload a node project and start a node on the runtime.
- **Node lifecycle:** Server-side APIs implement `create_node`, `compile_node`, and `run_node` (C FFI exposed to Rust) to manage node projects.
- **Per-node logging:** Node child processes have their `stdout` and `stderr` redirected to per-node log files: `/etc/nodes/<node_name>/build/logs/stdout.log` and `stderr.log` (changeable via `NODE_DIR`).

Getting the code
----------------
```bash
git clone https://github.com/f3fe-hash/FeO.git
cd FeO
```

Build the C runtime (recommended):
```bash
cmake -S . -B build
cmake --build build --target runtime -- -j 2
# then run the runtime binary
./build/feo_runtime
```

Build and run the Rust client/tooling (`fup`):
```bash
cd fup
cargo run -- <SUBCOMMAND> [ARGS]
```

Features & Usage
----------------
- **Upload a node project** (client-side `fup upload`): the client sends an `upload` handshake then streams files as framed packets. The server writes files under the node project directory (default base: `/etc/nodes/<project>`).
- **Start a node** (client-side `fup start <project>`): the client sends `start` then `START <project>`; the runtime will `create_node`, `compile_node` (runs `cargo build` for Rust nodes), and `run_node`. Output from the node goes into the per-node `logs` folder.
- **Framing details:** each data packet has a `Content-Length: <N>` header, CRLFCRLF, then exactly N bytes of payload. The payload format depends on the command (e.g., file upload metadata + file bytes).

Logs and Permissions
--------------------
- By default node projects and logs live under `/etc/nodes`. This requires write permissions for the runtime process. If you prefer a user-owned location, change `NODE_DIR` in `include/node.h` to a directory you own (for example `/home/feo/nodes`) and rebuild.
- Node `stdout` -> `.../build/logs/stdout.log` and `stderr` -> `.../build/logs/stderr.log`. Files are opened in append mode.

Notes for Developers
--------------------
- The codebase mixes C and Rust. High-level control and networking are implemented in Rust (`runtime` wrappers and `fup`), with core node lifecycle and runtime logic in C (`src/`).
- Important C functions exposed to Rust via FFI (in `runtime/src/c_link.rs`): `create_node`, `compile_node`, `run_node`, plus server socket helpers (`_listen_clients`, `_read_server`, `_write_server`).
- `fup` uses the `openssl` crate for TLS. Certificate handling is currently simple; in production you should verify certificates.

Troubleshooting
---------------
- If the runtime fails to create `/etc/nodes`, either run the runtime with appropriate permissions or edit `include/node.h` to a user-writable `NODE_DIR` and rebuild.
- If `fup` cannot connect, ensure the runtime is running and `keys/cert.pem` and `keys/key.pem` exist (see `make gen_keys` if present in the repo).

Contributing
------------
FeO is an evolving project. Contributions, bug reports, and feature requests are welcome.

Status
------
Work in progress. Expect breaking changes and unfinished features.