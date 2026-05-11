//! # `c_link`
//!
//! Rust FFI bindings for the FeO C backend.
//!
//! This module provides safe Rust wrappers around the native C API used by
//! FeO. It exposes functionality for:
//!
//! - TLS-enabled networking
//! - Client/server communication
//! - Process and node management
//! - Error handling through thread-local error codes
//!
//! The low-level C functions are exposed through `extern "C"` bindings,
//! while the public Rust API wraps them in safer abstractions such as
//! `Result<T, Error>`.
//!
//! ---
//!
//! # Module Layout
//!
//! The module is divided into two major sections:
//!
//! - Server bindings
//!   - Networking
//!   - TLS communication
//!   - Client callbacks
//!
//! - Node bindings
//!   - Process management
//!   - Node lifecycle handling
//!
//! ---
//!
//! # Error Handling
//!
//! Most wrapper functions return:
//!
//! ```rust
//! Result<T, Error>
//! ```
//!
//! The underlying C backend stores error codes in the thread-local global:
//!
//! ```rust
//! __global_err
//! ```
//!
//! Wrapper functions automatically translate these integer error codes into
//! Rust `Error` structures.
//!
//! Example:
//!
//! ```rust
//! match new_server("127.0.0.1", 8080)
//! {
//!     Ok(server) => println!("Server started"),
//!     Err(err) => println!("Error: {:?}", err),
//! }
//! ```
//!
//! ---
//!
//! # Documentation Style Guide
//!
//! This module uses Rust documentation comments (`///` and `//!`) so that
//! generated documentation works correctly with:
//!
//! ```bash
//! cargo doc --open
//! ```
//!
//! ## Inline Code
//!
//! Use backticks for:
//!
//! - Function names
//! - Types
//! - Constants
//! - Keywords
//! - Variables
//!
//! Example:
//!
//! ```text
//! `Result<T, Error>`
//! `CString`
//! `unsafe`
//! `ERR_FAILED_MALLOC`
//! ```
//!
//! ---
//!
//! ## Documentation Comment Types
//!
//! ### Module Documentation
//!
//! Uses:
//!
//! ```rust
//! //!
//! ```
//!
//! Example:
//!
//! ```rust
//! //! Networking bindings for FeO.
//! ```
//!
//! These appear at the top of the generated module documentation.
//!
//! ---
//!
//! ### Item Documentation
//!
//! Uses:
//!
//! ```rust
//! ///
//! ```
//!
//! Example:
//!
//! ```rust
//! /// Starts the server.
//! pub fn run_server()
//! ```
//!
//! These document:
//!
//! - Functions
//! - Structs
//! - Constants
//! - Fields
//! - Enums
//!
//! ---
//!
//! # Standard Documentation Format
//!
//! Functions in this module follow a consistent format:
//!
//! ```rust
//! /// Brief description.
//! ///
//! /// # Parameters
//! /// - `param`: Description.
//! ///
//! /// # Returns
//! /// - `Ok(T)` on success.
//! /// - `Err(Error)` on failure.
//! ///
//! /// # Errors
//! /// - `ERR_FAILED_MALLOC`
//! ```
//!
//! Common sections include:
//!
//! - `# Parameters`
//! - `# Returns`
//! - `# Errors`
//! - `# Safety`
//! - `# Notes`
//!
//! ---
//!
//! # Safety
//!
//! This module interfaces directly with native C code and therefore contains
//! `unsafe` blocks and raw pointers.
//!
//! Public wrappers attempt to provide safer abstractions, but misuse of raw
//! FFI bindings may still result in:
//!
//! - Undefined behavior
//! - Invalid memory access
//! - Use-after-free
//! - Data races
//!
//! Functions marked `unsafe` must follow all documented safety requirements.
//!
//! ---
//!
//! # Example
//!
//! ```rust
//! use c_link::{
//!     new_server,
//!     run_server,
//! };
//!
//! fn main()
//! {
//!     let server = new_server("127.0.0.1", 8080)
//!         .expect("Failed to create server");
//!
//!     run_server(&server)
//!         .expect("Failed to run server");
//! }
//! ```

// Allow unsused function / vairable warnings throughout the entire file.
// Cleans up compile log.
#![allow(unused)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_ushort};
use std::{thread, time::Duration};

use libc::size_t;

/*
    Error codes
*/

// System errors (0x0X)

/// No error present.
///
/// Default value for `__global_err`.
pub const ERR_OK: i32 = 0x00;

/// Memory allocation failure.
///
/// This is not limited to `malloc`; any allocation failure may trigger it.
pub const ERR_FAILED_MALLOC: i32 = 0x01;

// Process errors (0x1X)

/// `execlp` failed.
pub const ERR_EXECLP: i32 = 0x10;

/// Maximum process limit reached.
pub const ERR_MAX_PROC_REACHED: i32 = 0x11;

/// Failed to terminate a process.
pub const ERR_FAILED_TERMINATE: i32 = 0x12;

// Networking errors (0x2X)

/// OpenSSL failed to write data.
pub const ERR_INVALID_WRITE: i32 = 0x20;

/// OpenSSL failed to read data.
pub const ERR_INVALID_READ: i32 = 0x21;

/// Failed to create an OpenSSL context (`SSL_CTX *`).
pub const ERR_FAILED_CREATE_SSL_CTX: i32 = 0x22;

/// Failed to create an OpenSSL SSL object (`SSL *`).
pub const ERR_FAILED_CREATE_SSL: i32 = 0x23;

/// Certificate file (`cert.pem`) could not be found.
pub const ERR_NO_CERT_FILE: i32 = 0x24;

/// Private key file (`key.pem`) could not be found.
pub const ERR_NO_PRIV_FILE: i32 = 0x25;

/// Failed to create a socket.
pub const ERR_NO_SOCKET: i32 = 0x26;

/// Failed to bind a socket.
pub const ERR_FAILED_BIND: i32 = 0x27;

/// Failed to listen on a socket.
pub const ERR_FAILED_LISTEN: i32 = 0x28;

/// Failed to accept a client connection.
pub const ERR_FAILED_ACCEPT: i32 = 0x29;

/// Rust-side error structure.
#[derive(Debug)]
pub struct Error
{
    /// Error code from `__global_err`.
    err: i32,

    /// Human-readable error description.
    string: String,
}

unsafe extern "C"
{
    /// Thread-local global error code used by the C backend.
    ///
    /// # Safety
    /// - Thread-local (`__thread` in C).
    /// - Each thread has an independent error state.
    /// - May be overwritten by any subsequent FFI call.
    /// - Should be read immediately after a failing call.
    ///
    /// # Notes
    /// - Set only when an error occurs.
    /// - Undefined after successful calls unless manually reset.
    pub unsafe static mut __global_err: i32;
}

/*
    Server bindings
*/

#[repr(C)]
pub struct NetworkServer_t
{
    _private: [u8; 0],
}

/// Rust wrapper around `NetworkServer_t`.
pub struct Server
{
    /// Raw pointer to the underlying C server structure.
    pub server: *mut NetworkServer_t,
}

impl Default for Server
{
    fn default() -> Self
    {
        Self
        {
            server: std::ptr::null_mut(),
        }
    }
}

#[repr(C)]
pub struct NetworkClientConnection_t
{
    _private: [u8; 0],
}

/// Rust wrapper around `NetworkClientConnection_t`.
pub struct ClientConnection
{
    /// Raw pointer to the underlying client connection.
    conn: *mut NetworkClientConnection_t,
}

impl Default for ClientConnection
{
    fn default() -> Self
    {
        Self
        {
            conn: std::ptr::null_mut(),
        }
    }
}

unsafe extern "C"
{
    /// Creates and initializes a server instance.
    ///
    /// # Parameters
    /// - `ip`: IP address to bind the server to.
    /// - `port`: Port to listen on.
    ///
    /// # Errors
    /// - `ERR_FAILED_MALLOC`
    /// - `ERR_FAILED_CREATE_SSL_CTX`
    /// - `ERR_NO_CERT_FILE`
    /// - `ERR_NO_PRIV_FILE`
    /// - `ERR_NO_SOCKET`
    /// - `ERR_FAILED_BIND`
    /// - `ERR_FAILED_LISTEN`
    ///
    /// # Returns
    /// - Pointer to `NetworkServer_t` on success.
    /// - `NULL` on failure.
    ///
    /// # Safety
    /// - `ip` must point to a valid null-terminated C string.
    unsafe fn _listen_clients(
        ip: *const c_char,
        port: c_ushort,
    ) -> *mut NetworkServer_t;

    /// Starts the server loop.
    ///
    /// # Errors
    /// - `ERR_FAILED_ACCEPT`
    /// - `ERR_FAILED_CREATE_SSL`
    /// - `ERR_FAILED_MALLOC`
    ///
    /// # Notes
    /// - Runs the server in a separate process.
    unsafe fn _run_server(server: *mut NetworkServer_t);

    /// Stops the server.
    ///
    /// # Parameters
    /// - `server`: Pointer to a valid server instance.
    unsafe fn _stop_server(server: *mut NetworkServer_t);

    /// Writes data to a client connection.
    ///
    /// # Parameters
    /// - `conn`: Client connection.
    /// - `data`: Pointer to data buffer.
    /// - `len`: Buffer length.
    ///
    /// # Errors
    /// - `ERR_INVALID_WRITE`
    unsafe fn _write_server(
        conn: *mut NetworkClientConnection_t,
        data: *const c_char,
        len: size_t,
    );

    /// Reads data from a client connection.
    ///
    /// # Errors
    /// - `ERR_INVALID_READ`
    /// - `ERR_FAILED_MALLOC`
    ///
    /// # Returns
    /// - Pointer to a C string on success.
    /// - `NULL` on failure.
    unsafe fn _read_server(
        conn: *mut NetworkClientConnection_t,
    ) -> *mut c_char;

    /// Registers a callback function for new client connections.
    ///
    /// # Parameters
    /// - `server`: Server instance.
    /// - `func`: Callback function invoked per client.
    unsafe fn _set_server_response(
        server: *mut NetworkServer_t,
        func: extern "C" fn(*mut NetworkClientConnection_t) -> c_int,
    );

    /// Frees a C-allocated buffer.
    ///
    /// # Parameters
    /// - `buffer`: Buffer allocated by the C backend.
    unsafe fn _free_buffer(buffer: *mut c_char);
}

/// Creates a new server instance.
///
/// # Parameters
/// - `ip`: IP address to bind the server to.
/// - `port`: Port to listen on.
///
/// # Returns
/// - `Ok(Server)` on success.
/// - `Err(Error)` on failure.
pub fn new_server(ip: &str, port: u16) -> Result<Server, Error>
{
    unsafe
    {
        __global_err = ERR_OK;

        let ip_cstr: CString =
            CString::new(ip).expect("CString::new failed");

        let raw: *mut NetworkServer_t =
            _listen_clients(ip_cstr.as_ptr(), port);

        if raw.is_null()
        {
            let err: i32 = __global_err;

            let err_str: String = match err
            {
                ERR_FAILED_MALLOC =>
                {
                    String::from(
                        "listen_clients failed to allocate memory",
                    )
                }

                ERR_FAILED_CREATE_SSL_CTX =>
                {
                    String::from(
                        "listen_clients failed to create an OpenSSL context",
                    )
                }

                ERR_NO_CERT_FILE =>
                {
                    String::from(
                        "listen_clients could not find a certificate file",
                    )
                }

                ERR_NO_PRIV_FILE =>
                {
                    String::from(
                        "listen_clients could not find a private key file",
                    )
                }

                ERR_NO_SOCKET =>
                {
                    String::from(
                        "listen_clients failed to create a socket",
                    )
                }

                ERR_FAILED_BIND =>
                {
                    String::from(
                        "listen_clients failed to bind the socket",
                    )
                }

                ERR_FAILED_LISTEN =>
                {
                    String::from(
                        "listen_clients failed to listen for clients",
                    )
                }

                _ =>
                {
                    String::from(
                        "listen_clients returned an unknown error",
                    )
                }
            };

            return Err(Error
            {
                err,
                string: err_str,
            });
        }

        Ok(Server { server: raw })
    }
}

/// Starts a server instance.
///
/// # Parameters
/// - `server`: Server returned by `new_server`.
///
/// # Returns
/// - `Ok(())` on success.
/// - `Err(Error)` on failure.
pub fn run_server(server: &Server) -> Result<(), Error>
{
    unsafe
    {
        __global_err = ERR_OK;

        _run_server(server.server);

        thread::sleep(Duration::from_millis(10));

        let err: i32 = __global_err;

        if err != ERR_OK
        {
            let err_str: String = match err
            {
                ERR_FAILED_ACCEPT =>
                {
                    String::from(
                        "run_server failed to accept a client",
                    )
                }

                ERR_FAILED_CREATE_SSL =>
                {
                    String::from(
                        "run_server failed to create an SSL object",
                    )
                }

                ERR_FAILED_MALLOC =>
                {
                    String::from(
                        "run_server failed to allocate memory",
                    )
                }

                _ =>
                {
                    String::from(
                        "run_server returned an unknown error",
                    )
                }
            };

            return Err(Error
            {
                err,
                string: err_str,
            });
        }
    }

    Ok(())
}

/// Stops a running server.
///
/// # Parameters
/// - `server`: Server instance to stop.
pub fn stop_server(server: &Server)
{
    unsafe
    {
        _stop_server(server.server);
    }
}

/// Reads data from a client connection.
///
/// # Parameters
/// - `conn`: Valid client connection.
///
/// # Returns
/// - `Ok(String)` containing the received data.
/// - `Err(Error)` on failure.
pub fn read_server(
    conn: &ClientConnection,
) -> Result<String, Error>
{
    unsafe
    {
        __global_err = ERR_OK;

        let request_ptr: *mut c_char =
            _read_server(conn.conn);

        if request_ptr.is_null()
        {
            let err: i32 = __global_err;

            let err_str: String = match err
            {
                ERR_INVALID_READ =>
                {
                    String::from(
                        "read_server failed to read data from the client",
                    )
                }

                ERR_FAILED_MALLOC =>
                {
                    String::from(
                        "read_server failed to allocate memory",
                    )
                }

                _ =>
                {
                    String::from(
                        "read_server returned an unknown error",
                    )
                }
            };

            return Err(Error
            {
                err,
                string: err_str,
            });
        }

        let c_str: &CStr = CStr::from_ptr(request_ptr);

        let result: String =
            c_str.to_string_lossy().into_owned();

        _free_buffer(request_ptr);

        Ok(result)
    }
}

/// Writes data to a client connection.
///
/// # Parameters
/// - `conn`: Valid client connection.
/// - `data`: Data to send.
///
/// # Returns
/// - `Ok(())` on success.
/// - `Err(Error)` on failure.
pub fn write_server(
    conn: &ClientConnection,
    data: String,
) -> Result<(), Error>
{
    unsafe
    {
        let c_string: CString =
            CString::new(data).expect("CString::new failed");

        let ptr: *const c_char = c_string.as_ptr();

        let len: size_t = c_string.as_bytes().len();

        __global_err = ERR_OK;

        _write_server(conn.conn, ptr, len);

        let err: i32 = __global_err;

        if err == ERR_INVALID_WRITE
        {
            return Err(Error
                {
                    err,
                    string: String::from(
                        "write_server failed to write data to the client",
                    ),
            });
        }
    }

    Ok(())
}

/// Global client callback handler.
///
/// Used internally by `callback_trampoline`.
static mut CLIENT_HANDLER: Option<fn(ClientConnection) -> Result<(), Error>> =
    None;

/// Internal callback trampoline.
///
/// Converts a Rust callback into a C-compatible callback.
///
/// # Notes
/// - Supports only one global callback handler.
extern "C" fn _callback_trampoline(
    conn_: *mut NetworkClientConnection_t,
) -> c_int
{
    unsafe
    {
        let conn_rs: ClientConnection = ClientConnection
        {
            conn: conn_,
        };

        match CLIENT_HANDLER
        {
            Some(func) =>
            {
                match func(conn_rs)
                {
                    Ok(()) => 0,
                    Err(err) => err.err,
                }
            }

            None => -1,
        }
    }
}

/// Registers a client callback handler.
///
/// # Parameters
/// - `server`: Server instance.
/// - `func`: Callback invoked for each client connection.
pub fn add_client_handle(
    server: &Server,
    func: fn(ClientConnection) -> Result<(), Error>,
)
{
    unsafe
    {
        CLIENT_HANDLER = Some(func);

        _set_server_response(
            server.server,
            _callback_trampoline,
        );
    }
}

/*
    Node bindings
*/

/// Maximum number of tracked processes.
pub const MAX_NUM_PROCS: usize = 128;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Node_t {
    pub pid: libc::pid_t,
    pub active: c_int,
    pub name: *mut c_char,
    pub name_len: c_int,
}

/// Rust wrapper around `Node_t`.
pub struct Node {
    node: *mut Node_t,
}

unsafe extern "C"
{
    /// Global process table managed by the C backend.
    unsafe static mut procs: [Node_t; MAX_NUM_PROCS];

    /// Initializes the node API.
    unsafe fn _init_nodes();

    /// Creates a node instance.
    ///
    /// # Parameters
    /// - `name`: Node name.
    ///
    /// # Errors
    /// - `ERR_FAILED_MALLOC`
    ///
    /// # Returns
    /// - Pointer to `Node_t` on success.
    /// - `NULL` on failure.
    unsafe fn _create_node(
        name: *const c_char,
    ) -> *mut Node_t;

    /// Kills a node process.
    unsafe fn _kill_node(node: *mut Node_t);

    /// Runs a node process.
    ///
    /// # Errors
    /// - `ERR_MAX_PROC_REACHED`
    unsafe fn _run_node(node: *mut Node_t);

    /// Frees a node instance.
    unsafe fn _free_node(node: *mut Node_t);

    /// Restarts a node process.
    unsafe fn _restart_node(node: *mut Node_t);

    /// Reaps completed processes.
    ///
    /// # Errors
    /// - `ERR_FAILED_MALLOC`
    ///
    /// # Returns
    /// - Pointer to a PID array on success.
    /// - `NULL` on failure.
    unsafe fn _reap_processes() -> *mut c_int;
}

/// Creates a new node.
///
/// # Parameters
/// - `name`: Node name.
///
/// # Returns
/// - `Ok(Node)` on success.
/// - `Err(Error)` on failure.
pub fn create_node(name: String) -> Result<Node, Error>
{
    unsafe
    {
        __global_err = ERR_OK;

        let name: CString =
            CString::new(name).expect("CString::new failed");

        let node: *mut Node_t =
            _create_node(name.as_ptr());

        if node.is_null()
        {
            let err: i32 = __global_err;

            if err == ERR_FAILED_MALLOC
            {
                return Err(Error
                {
                    err,
                    string: String::from(
                        "create_node failed to allocate memory",
                    ),
                });
            }
        }

        Ok(Node { node })
    }
}

/// Kills a node.
///
/// # Parameters
/// - `node`: Node to terminate.
pub fn kill_node(node: Node)
{
    unsafe
    {
        _kill_node(node.node);
    }
}

/// Starts a node.
///
/// # Parameters
/// - `node`: Node to run.
///
/// # Returns
/// - `Ok(())` on success.
/// - `Err(Error)` on failure.
pub fn run_node(node: Node) -> Result<(), Error>
{
    unsafe
    {
        __global_err = ERR_OK;

        _run_node(node.node);

        let err: i32 = __global_err;

        if err != ERR_OK
        {
            if err == ERR_MAX_PROC_REACHED
            {
                return Err(Error
                {
                    err,
                    string: String::from(
                        "run_node failed to allocate a new process",
                    ),
                });
            }
        }
    }

    Ok(())
}

/// Frees a node instance.
///
/// # Parameters
/// - `node`: Node to free.
pub fn free_node(node: Node)
{
    unsafe
    {
        _free_node(node.node);
    }
}

/// Restarts a node.
///
/// # Parameters
/// - `node`: Node to restart.
pub fn restart_node(node: Node)
{
    unsafe
    {
        _restart_node(node.node);
    }
}

/// Reaps completed processes.
///
/// # Returns
/// - `Ok(Vec<i32>)` containing process IDs.
/// - `Err(Error)` on failure.
pub fn reap_procs() -> Result<Vec<i32>, Error>
{
    unsafe
    {
        __global_err = ERR_OK;

        let c_arr: *mut i32 = _reap_processes();

        let data: Vec<i32> = Vec::from_raw_parts(
            c_arr,
            MAX_NUM_PROCS,
            MAX_NUM_PROCS,
        );

        let err: i32 = __global_err;

        if err != ERR_OK
        {
            if err == ERR_FAILED_MALLOC
            {
                return Err(Error
                {
                    err,
                    string: String::from(
                        "reap_procs failed to allocate memory",
                    ),
                });
            }
        }

        Ok(data)
    }
}

