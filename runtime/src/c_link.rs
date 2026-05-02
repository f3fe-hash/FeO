use std::os::raw::{c_char, c_ushort, c_int};
use libc::size_t;



/*
    Server bindings
*/



#[repr(C)]
pub struct NetworkServer_t
{
    _private: [u8; 0],
}

#[repr(C)]
pub struct NetworkClientConnection_t
{
    _private: [u8; 0],
}

unsafe extern "C"
{
    pub unsafe fn listen_clients(ip: *const c_char, port: c_ushort) -> *mut NetworkServer_t;
    pub unsafe fn run_server(server: *mut NetworkServer_t);
    pub unsafe fn stop_server(server: *mut NetworkServer_t);
    pub unsafe fn write_server(conn: *mut NetworkClientConnection_t, data: *const c_char, len: size_t);
    pub unsafe fn read_server(conn: *mut NetworkClientConnection_t) -> *mut c_char;
    pub unsafe fn set_server_response(
        server: *mut NetworkServer_t,
        func: extern "C" fn(*mut NetworkClientConnection_t) -> c_int
    );

    pub unsafe fn free_buffer(buffer: *mut c_char);
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
    pub unsafe static mut procs: [Node_t; MAX_NUM_PROCS];

    pub unsafe fn init_nodes() -> c_int;
    pub unsafe fn create_node(name: *const c_char) -> *mut Node_t;
    pub unsafe fn kill_node(node: *mut Node_t) -> c_int;
    pub unsafe fn run_node(node: *mut Node_t) -> c_int;
    pub unsafe fn free_node(node: *mut Node_t) -> c_int;
    pub unsafe fn restart_node(node: *mut Node_t) -> c_int;
    pub unsafe fn reap_processes() -> *mut c_int;
}

// C macros
pub const ERR_OK: i32 = 0;
pub const ERR_EXECLP: i32 =  1;
pub const ERR_MAX_PROC_REACHED: i32 = 2;
