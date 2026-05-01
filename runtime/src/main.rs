// Custom C linkage module
mod c_link;
use c_link::{
    listen_clients,
    set_server_response,
    run_server,
    stop_server,
    write_server,
    NetworkClientConnection_t
};

use std::ffi::CString;
use std::{thread, time::Duration};

// Client handler
extern "C" fn handle_client(conn: *mut NetworkClientConnection_t) -> i32
{
    let response = CString::new("testing").unwrap();
    println!("Client connected!");
    unsafe
    {
        write_server(conn, response.as_ptr(), response.as_bytes().len());
    }
    0
}

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
        println!("Server is running.");

        run_server(server);
        thread::sleep(Duration::from_secs(10));
        stop_server(server);
        println!("Server has been stopped.");
    }
}