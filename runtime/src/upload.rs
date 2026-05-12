use std::fs;
use std::path::Path;

use crate::c_link::{
    read_server,
    ClientConnection,
    Error
};

const NODE_ROOT: &str = "/home/feo/nodes";

pub fn handle_upload(
    conn: &ClientConnection
) -> Result<(), Error>
{
    let upload_str: String =
        read_server(conn)?;

    let parts: Vec<&str> =
        upload_str.splitn(2, ' ').collect();

    let project_name: &str = parts[1].trim();

    let base_dir = Path::new(NODE_ROOT).join(project_name);

    fs::create_dir_all(&base_dir).unwrap();

    loop {
        let msg_str: String = read_server(conn)?;

        if msg_str == "END" {
            break;
        }

        let msg: Vec<u8> = msg_str.into_bytes();

        let newline_pos = match msg.iter().position(|&b| b == b'\n') {
            Some(p) => p,
            None => continue,
        };

        let header = String::from_utf8_lossy(&msg[..newline_pos]);

        let file_data = &msg[newline_pos + 1..];

        let header_parts: Vec<&str> = header.splitn(2, ' ').collect();

        if header_parts[0] != "FILE" {
            continue;
        }

        let relative_path = header_parts[1];

        if relative_path.contains("..") {
            continue;
        }

        let full_path = base_dir.join(relative_path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(full_path, file_data).unwrap();
    }

    Ok(())
}