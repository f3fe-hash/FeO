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
        write_server(&conn, "r".to_string())?;
        handle_upload(&conn)?;
    }

    else if request.trim() == "start"
    {
        write_server(&conn, "r".to_string())?;

        // Expecting: "START <project_name>\n"
        let start_str: String = read_server(&conn)?;
        let parts: Vec<&str> = start_str.splitn(2, ' ').collect();
        if parts.len() < 2 {
            write_server(&conn, "err Invalid START format".to_string())?;
            return Ok(());
        }

        let project_name = parts[1].trim();

        // Create, compile, and run the node via C bindings.
        let node = match c_link::create_node(project_name.to_string()) {
            Ok(n) => n,
            Err(e) => {
                write_server(&conn, format!("err create_node: {}", e.string))?;
                return Ok(());
            }
        };

        if let Err(e) = c_link::compile_node(&node) {
            write_server(&conn, format!("err compile_node: {}", e.string))?;
            c_link::free_node(node);
            return Ok(());
        }

        if let Err(e) = c_link::run_node(node) {
            write_server(&conn, format!("err run_node: {}", e.string))?;
            return Ok(());
        }

        write_server(&conn, "ok".to_string())?;
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