// Custom C linkage module
mod c_link;
use c_link::{
    listen_clients,
    set_server_response,
    run_server,
    stop_server,
    read_server,
    write_server,
    NetworkClientConnection_t
};

use std::ffi::{CString};
use std::{thread, time::Duration};
use std::os::raw::{c_int};

// Client handler for FUP
// Warning: Runs in separate process
extern "C" fn handle_client(conn: *mut NetworkClientConnection_t) -> c_int
{
    let request = read_server(conn);
    // Uploading a node
    if request == "upload"
    {
        write_server(conn, "r".to_string());
        let response: String = read_server(conn);
        
    }

    write_server(conn, "testing".to_string());
    0
}

fn main()
{
    let ip: CString = CString::new("127.0.0.1").unwrap();

    unsafe
    {
        let server: *mut c_link::NetworkServer_t = listen_clients(ip.as_ptr(), 1234);

        if server.is_null()
        {
            panic!("Server creation failed");
        }

        set_server_response(server, handle_client);
        println!("Server is running.");

        run_server(server);
        thread::sleep(Duration::from_secs(100));
        stop_server(server);
        println!("Server has been stopped.");
    }
}