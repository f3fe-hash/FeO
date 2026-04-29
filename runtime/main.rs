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
use std::os::raw::{c_char, c_short, c_int};

extern "C"
{
    fn listen_clients(ip: *const c_char, port: c_short) -> *mut NetworkServer_t;
    fn run_server(server: *mut NetworkServer_t);
    fn stop_server(server: *mut NetworkServer_t);

    fn set_server_response(
        server: *mut NetworkServer_t,
        func: extern "C" fn(*mut NetworkClientConnection_t) -> c_int
    );
}

extern "C" fn handle_client(conn: *mut NetworkClientConnection_t) -> i32
{
    println!("Client connected!");
    0
}

use std::ffi::CString;
use std::{thread, time::Duration};

fn main()
{
    let ip = CString::new("127.0.0.1").unwrap();

    unsafe
    {
        let server = listen_clients(ip.as_ptr(), 8080);

        if server.is_null()
        {
            panic!("Server creation failed");
        }

        set_server_response(server, handle_client);

        run_server(server);
        thread::sleep(Duration::from_secs(10));
        stop_server(server);
    }
}