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
    ClientConnection,
    Error
};

mod upload;
use upload::handle_upload;
use std::{thread, time::Duration};

fn handle_client(
    conn: ClientConnection
) -> Result<(), Error>
{
    let request: String =
    read_server(&conn)?;

    println!("REQUEST = {:?}", request);

    if request.trim() == "upload"
    {
        println!("Sending 'r' to client");
        write_server(&conn, "r".to_string())?;
        println!("Sent 'r' to client");

        println!("Entering handle_upload");
        handle_upload(&conn)?;
        println!("handle_upload returned");
    }

    Ok(())
}

fn main()
{
    let server: Server = new_server("0.0.0.0", 1234).expect("");

    add_client_handle(&server, handle_client);
    println!("Server is running.");

    run_server(&server).expect("");
    thread::sleep(Duration::from_secs(100));
    stop_server(&server);
    println!("Server has been stopped.");
}