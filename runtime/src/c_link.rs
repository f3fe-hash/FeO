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

use std::os::raw::{c_char, c_ushort, c_int};
use libc::size_t;

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