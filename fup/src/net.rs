use std::net::TcpStream;
use std::io::Write;

pub fn write_packet(
    stream: &mut TcpStream,
    data: &[u8]
) -> std::io::Result<()>
{
    let header: String =
        format!("Content-Length: {}\r\n\r\n", data.len());

    stream.write_all(header.as_bytes())?;
    stream.write_all(data)?;

    Ok(())
}