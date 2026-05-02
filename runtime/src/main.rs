// Custom C linkage module
mod c_link;
use c_link::{
    listen_clients,
    set_server_response,
    run_server,
    stop_server,
    read_server,
    write_server,
    free_buffer,
    NetworkClientConnection_t
};

use std::ffi::{CString, CStr};
use std::{thread, time::Duration};
use std::os::raw::{c_int};

// Client handler
// Warning: Runs in separate process
extern "C" fn handle_client(conn: *mut NetworkClientConnection_t) -> c_int
{
    unsafe
    {
        let request_ptr: *mut i8 = read_server(conn);

        if !request_ptr.is_null()
        {
            let request = CStr::from_ptr(request_ptr);
            println!("Client sent: {}", request.to_string_lossy());
            free_buffer(request_ptr);
        }

        let response: CString = CString::new("testing").unwrap();
        write_server(conn, response.as_ptr(), response.as_bytes().len());
    }
    0
}

fn main()
{
    let ip: CString = CString::new("127.0.0.1").unwrap();

    unsafe
    {
        let server: *mut c_link::NetworkServer_t = listen_clients(ip.as_ptr(), 8080);

        if server.is_null()
        {
            panic!("Server creation failed");
        }

        set_server_response(server, handle_client);
        println!("Server is running.");

        run_server(server);
        thread::sleep(Duration::from_secs(10));
        stop_server(server);
        println!("Server has been stopped.");
    }
}