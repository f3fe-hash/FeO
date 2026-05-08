use std::ffi::{CStr, CString};

use std::{thread, time::Duration};
use std::os::raw::{c_char, c_ushort, c_int};
use libc::size_t;


// C macros

// System errors (0x0X)
pub const ERR_OK: i32 = 0x00; // No error present. Default for `__global_err`.
pub const ERR_FAILED_MALLOC: i32 = 0x01; // C allocation failed. No specifically `malloc` failure. Any allocation failure can cause this.

// Process errors (0x1X)
pub const ERR_EXECLP: i32 = 0x10; // `execlp` failure
pub const ERR_MAX_PROC_REACHED: i32 = 0x11; // The maximum process limit has been reached.
pub const ERR_FAILED_TERMINATE: i32 = 0x12; // C failed to terminate a process.

// Networking errors (0x2X)
pub const ERR_INVALID_WRITE: i32 = 0x20; // OpenSSL failed to write data
pub const ERR_INVALID_READ: i32 = 0x21; // OpenSSL failed to read data.
pub const ERR_FAILED_CREATE_SSL_CTX: i32 = 0x22; // OpenSSL failed to create a context (`SSL_CTX *`)
pub const ERR_FAILED_CREATE_SSL: i32 = 0x23; // OpenSSL failed to create an SSL (`SSL *`)
pub const ERR_NO_CERT_FILE: i32 = 0x24; // OpenSSL couldn't find a certificate file (cert.pem).
pub const ERR_NO_PRIV_FILE: i32 = 0x25; // OpenSSL coundn't find a private key file (key.pem).
pub const ERR_NO_SOCKET: i32 = 0x26; // C failed to create a socket.
pub const ERR_FAILED_BIND: i32 = 0x27; // C failed to bind an address to a socket.
pub const ERR_FAILED_LISTEN: i32 = 0x28; // C failed to start listening on a socket.
pub const ERR_FAILED_ACCEPT: i32 = 0x29; // C (or OpenSSL) failed to accept a client.

unsafe extern "C"
{
    /// Global error code set by the last C operation (thread-local).
    ///
    /// # Safety
    /// - This value is thread-local (`__thread` in C).
    /// - Each thread has its own independent error state.
    /// - May be overwritten by any subsequent FFI call in the same thread.
    /// - Must be read immediately after a failing call.
    ///
    /// # Behavior
    /// - Set only when an error occurs.
    /// - Undefined if read after a successful call unless reset manually.
    pub unsafe static mut __global_err: i32;
}


/*
    Server bindings
*/

//fn def_handle(_conn: ClientConnection) -> i32 {0}


#[repr(C)]
pub struct NetworkServer_t
{
    _private: [u8; 0],
}

#[derive(Default)]
pub struct Server
{
    pub server: *mut NetworkServer_t,
    //pub handle: Option<fn(ClientConnection) -> i32>,
}

#[repr(C)]
pub struct NetworkClientConnection_t
{
    _private: [u8; 0],
}

#[derive(Default)]
pub struct ClientConnection
{
    conn: *mut NetworkClientConnection_t,
}

unsafe extern "C"
{
    /// C interface for initiating server.
    /// Sets `__global_err` on failure.
    /// 
    /// 
    /// # Error values
    /// - ERR_FAILED_MALLOC: C failed to allocate memory
    /// - ERR_FAILED_CREATE_SSL_CTX: C failed to create an OpenSSL context.
    /// - ERR_NO_CERT_FILE: OpenSSL failed at opening certificate file.
    /// - ERR_NO_PRIV_FILE: OpenSSL failed at opening private key file.
    /// - ERR_NO_SOCKET: C failed opening a socket.
    /// - ERR_FAILED_BIND: C failed to bind ip address & port to socket
    /// - ERR_FAILED_LISTEN: C failed to start listening for incoming
    /// connections on that port
    /// 
    /// # Returns
    /// - NetworkServer_t structure on success
    /// - NULL on failure
    /// 
    /// # Notes
    /// - Errors are uncommon in typical configurations, but callers
    /// should still check `__global_err` if failure handling is required.
    unsafe fn _listen_clients(ip: *const c_char, port: c_ushort) -> *mut NetworkServer_t;

    /// C interface for running a server.
    /// Sets `__global_err` on failure.
    /// 
    /// 
    /// # Error values
    /// - ERR_FAILED_ACCEPT: C (or OpenSSL) failed to accept a client connection
    /// - ERR_FAILED_CREATE_SSL: OpenSSL failed to create an SSL
    /// - ERR_FAILED_MALLOC: C failed to allocate memory
    /// 
    /// # Returns
    /// - None
    /// 
    /// # Notes
    /// - Creates separate process. If error log
    /// desired, monitoring of `__global_err` required.
    unsafe fn _run_server(server: *mut NetworkServer_t);

    /// C interface for stopping a server.
    /// 
    /// 
    /// # Error values
    /// - None
    /// 
    /// # Returns
    /// - None
    unsafe fn _stop_server(server: *mut NetworkServer_t);

    /// C interface for writing to a client.
    /// Sets `__global_err` on failure.
    /// 
    /// 
    /// # Error values
    /// - ERR_INVALID_WRITE: OpenSSL failed to write data to client.
    /// 
    /// # Returns
    /// - None
    unsafe fn _write_server(conn: *mut NetworkClientConnection_t, data: *const c_char, len: size_t);

    /// C interface for reading from a client.
    /// Sets `__global_err` on failure.
    /// 
    /// 
    /// # Error values
    /// - ERR_INVALID_READ: OpenSSL failed to read data from client
    /// - ERR_FAILED_MALLOC: C failed to allocate memory
    /// 
    /// # Returns
    /// - Pointer to c-style string on success
    /// - NULL on failure
    unsafe fn _read_server(conn: *mut NetworkClientConnection_t) -> *mut c_char;

    /// C interface for setting server response function.
    ///
    /// 
    /// # Error values
    /// - None
    /// 
    /// # Returns
    /// - None
    unsafe fn _set_server_response(
        server: *mut NetworkServer_t,
        func: extern "C" fn(*mut NetworkClientConnection_t) -> c_int
    );

    /// C interface for freeing C allocated buffers.
    ///
    /// 
    /// # Error values
    /// - None
    /// 
    /// # Returns
    /// - None
    unsafe fn free_buffer(buffer: *mut c_char);
}

/// Rust `_listen_clients` wrapper
/// 
/// 
/// # Inputs
/// - ip: IP string of local machine to bind server to.
/// - port: Port to bind server to.
/// 
/// # Outputs
/// - Returns non-empty client response string on sucess.
/// - Returns empty string and prints error on `_read_server` failure.
pub fn new_server(ip: &str, port: u16) -> Server
{
    unsafe 
    {
        let ip_cstr: CString = CString::new(ip).expect("CString::new failed");
        let raw: *mut NetworkServer_t = _listen_clients(ip_cstr.as_ptr(), port);
        if raw.is_null()
        {
            let err: i32 = __global_err; // Preserve `__global_err` state

            // match the error.
            match err
            {
                ERR_FAILED_MALLOC => eprintln!("listen_clients failed to allocate memory."),
                ERR_FAILED_CREATE_SSL_CTX => eprintln!("listen_clients failed to create an OpenSSL context."),
                ERR_NO_CERT_FILE => eprintln!("listen_clients couldn't find a certificate file."),
                ERR_NO_PRIV_FILE => eprintln!("listen_clients couldn't find a private ket file."),
                ERR_NO_SOCKET => eprintln!("listen_clients couldn't create a socket."),
                ERR_FAILED_BIND => eprintln!("listen_clients couldn't bind address to socket"),
                ERR_FAILED_LISTEN => eprintln!("listen_clients failed to listen for clients"),

                _ => eprintln!("listen_clients had an unknown error"),
            }
            
            return Server::default();
        }

        Server
        {
            server: raw
        }
    }
}

/// Rust `_run_server` wrapper
/// 
/// 
/// # Inputs
/// - server: Rust server struct generated by `new_server`
/// 
/// # Outputs
/// - None
pub fn run_server(server: &Server)
{
    unsafe
    {
        __global_err = ERR_OK;
        _run_server(server.server);
        thread::sleep(Duration::from_millis(10)); // Wait a few millis for immediate crash.
        let err = __global_err; // Preserve `__global_err` state
        if err != ERR_OK
        {
            match err
            {
                ERR_FAILED_ACCEPT => eprintln!("run_server failed to accept a client"),
                ERR_FAILED_CREATE_SSL => eprintln!("run_server failed to create an ssl"),
                ERR_FAILED_MALLOC => eprintln!("run_server failed to allocate memory"),

                _ => eprintln!("run_server had an unknown error"),
            }
        }
    }
}

pub fn stop_server(server: &Server)
{
    unsafe
    {
        _stop_server(server.server);
    }
}

/// Rust `_read_server` wrapper
/// 
/// 
/// # Inputs
/// - conn: A valid connection of a server created by `listen_clients`.
/// 
/// # Outputs
/// - Returns non-empty client response string on sucess.
/// - Returns empty string and prints error on `_read_server` failure.
pub fn read_server(conn: &ClientConnection) -> String
{
    unsafe 
    {
        let request_ptr: *mut c_char = _read_server(conn.conn);

        if request_ptr.is_null()
        {
            let err: i32 = __global_err; // Preserve `__global_err` state
            if err == ERR_INVALID_READ
            {
                eprintln!("read_server error: Failed to read data from client.");
            }
            else if err == ERR_FAILED_MALLOC
            {
                eprintln!("read_server error: Failed to allocate memory");
            }
            return String::new();
        }

        // Borrow C string (DO NOT take ownership)
        let c_str: &CStr = CStr::from_ptr(request_ptr);

        let result: String = c_str.to_string_lossy().into_owned();

        // Free C buffer
        free_buffer(request_ptr);

        result
    }
}

/// Rust `_write_server` wrapper
/// 
/// 
/// # Inputs
/// - conn: A valid connection of a server created by `listen_clients`.
/// - data: A string containing data to send to client.
/// 
/// # Outputs
/// - No return value.
/// - Prints error on `_write_server` error.
pub fn write_server(conn: &ClientConnection, data: String)
{
    unsafe
    {
        // Convert to CString (ensures null-termination and no interior nulls)
        let c_string: CString = CString::new(data).expect("CString::new failed");

        let ptr: *const c_char = c_string.as_ptr();
        let len: size_t = c_string.as_bytes().len();

        __global_err = ERR_OK; // Reset `__global_err`
        _write_server(conn.conn, ptr, len);
        let err: i32 = __global_err; // Preserve `__global_err` state
        if err == ERR_INVALID_WRITE
        {
            eprintln!("write_server error: Failed to write data to client (0x{:X})", err);
        }
    }
}

/// Client handler for `callback_trampoline`` method
static mut CLIENT_HANDLER: Option<fn(ClientConnection) -> i32> = None;

/// # Callback trampoline
/// 
/// Private member to convert Rust callback function to C callback
/// function. Uses 1 global client handler, limiting individual program
/// instances to 1 client handler each.
extern "C" fn callback_trampoline(conn_: *mut NetworkClientConnection_t) -> c_int
{
    unsafe
    {
        let conn_rs: ClientConnection = ClientConnection
        {
            conn: conn_
        };

        match CLIENT_HANDLER
        {
            Some(func) => func(conn_rs) as c_int,
            None => -1
        }
    }
}

/// # `_set_server_response` wrapper
/// 
/// # Inputs:
/// - server: Rust server struct.
/// - func: Functiion to call for each client the server encounters.
/// 
/// Outputs:
/// - None
pub fn add_client_handle(server: &mut Server, func: fn(ClientConnection) -> i32)
{
    unsafe
    {
        CLIENT_HANDLER = Some(func);

        _set_server_response(server.server, callback_trampoline);
    }
}

/*
    Node bindings
*/



pub const MAX_NUM_PROCS: usize = 128;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Node_t
{
    pub pid: libc::pid_t,
    pub active: c_int,
    pub name: *mut c_char,
    pub name_len: c_int,
}

unsafe extern "C"
{
    /// A list of all currently running FeO processes.
    pub unsafe static mut procs: [Node_t; MAX_NUM_PROCS];

    /// C interface for initializing the node API.
    /// 
    /// 
    /// # Error values
    /// - None
    /// 
    /// # Returns
    /// - None
    pub unsafe fn init_nodes();

    /// C interface for creating a node.
    /// Sets `__global_err` on failure.
    /// 
    /// 
    /// # Error values
    /// - ERR_FAILED_MALLOC: C failed to allocate memory
    /// 
    /// # Returns
    /// - Pointer to a node
    /// - NULL on failure
    pub unsafe fn create_node(name: *const c_char) -> *mut Node_t;

    /// C interface for killing a node.
    /// 
    /// 
    /// # Error values
    /// - None
    /// 
    /// # Returns
    /// - None
    pub unsafe fn kill_node(node: *mut Node_t);

    /// C interface for killing a node.
    /// Sets `__global_err` on failure.
    /// 
    /// 
    /// # Error values
    /// - ERR_MAX_PROC_LIMIT_REACHED: Too many processes are registered (FeO processes, not linux processes)
    /// 
    /// # Returns
    /// - None
    pub unsafe fn run_node(node: *mut Node_t);

    /// C interface for killing a node.
    /// 
    /// 
    /// # Error values
    /// - None
    /// 
    /// # Returns
    /// - None
    pub unsafe fn free_node(node: *mut Node_t);

    /// C interface for killing a node.
    /// 
    /// 
    /// # Error values
    /// - None
    /// 
    /// # Returns
    /// - None
    pub unsafe fn restart_node(node: *mut Node_t);

    /// C interface for killing a node.
    /// Sets `__global_err` on failure.
    /// 
    /// 
    /// # Error values
    /// - ERR_FAILED_MALLOC: C failed to allocate memory
    /// 
    /// # Returns
    /// - A list of pids of running processes if sucessful
    /// - NULL on failure
    pub unsafe fn reap_processes() -> *mut c_int;
}