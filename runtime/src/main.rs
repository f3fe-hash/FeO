// Custom C linkage module
mod c_link;
use c_link::{
    new_server,
    add_client_handle,
    run_server,
    stop_server,
    read_server,
    write_server,
    Server,
    ClientConnection
};

use std::{thread, time::Duration};

// Client handler for FUP
// Warning: Runs in separate process
fn handle_client(conn: ClientConnection) -> i32
{
    let request: String = read_server(&conn).expect("");
    // Uploading a node
    if request == "upload"
    {
        write_server(&conn, "r".to_string()).expect("");
        let _response: String = read_server(&conn).expect("");
        
    }

    write_server(&conn, "testing".to_string()).expect("");
    0
}

fn main()
{
    let mut server: Server = new_server("127.0.0.1", 1234).expect("");

    add_client_handle(&mut server, handle_client);
    println!("Server is running.");

    run_server(&server).expect("");
    thread::sleep(Duration::from_secs(100));
    stop_server(&server);
    println!("Server has been stopped.");
}