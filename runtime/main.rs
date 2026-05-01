mod c_link
use c_link::*;
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