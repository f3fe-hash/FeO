use std::net::TcpStream;
use std::io::{Write, Read};
use openssl::ssl::{SslConnector, SslMethod, SslStream, SslVerifyMode};

pub type TlsStream = SslStream<TcpStream>;

pub fn connect_tls(addr: &str, domain: &str) -> std::io::Result<TlsStream>
{
    let stream = TcpStream::connect(addr)?;
    let mut builder = SslConnector::builder(SslMethod::tls()).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    builder.set_verify(SslVerifyMode::NONE);
    let connector = builder.build();
    let ssl_stream = connector.connect(domain, stream).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(ssl_stream)
}

pub fn write_packet(
    stream: &mut TlsStream,
    data: &[u8]
) -> std::io::Result<()> {
    let header: String = format!("Content-Length: {}\r\n\r\n", data.len());

    stream.write_all(header.as_bytes())?;
    stream.write_all(data)?;

    Ok(())
}

pub fn read_packet(stream: &mut TlsStream) -> std::io::Result<Vec<u8>>
{
    let mut buf: Vec<u8> = Vec::new();
    let mut header_end: Option<usize> = None;

    let mut tmp: [u8; 1024] = [0u8; 1024];
    while header_end.is_none()
    {
        let n = stream.read(&mut tmp)?;
        if n == 0 { return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof)); }
        buf.extend_from_slice(&tmp[..n]);
        if buf.len() >= 4
        {
            for i in 0..= buf.len() - 4
            {
                if &buf[i..i + 4] == b"\r\n\r\n" {
                    header_end = Some(i + 4);
                    break;
                }
            }
        }
        if buf.len() > 10_000_000
        { // sanity limit
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "header too large"));
        }
    }

    let he: usize = header_end.unwrap();
    let header: &[u8] = &buf[..he];
    let cl_str: String = match header.windows(15).position(|w| w.eq(b"Content-Length:"))
    {
        Some(pos) =>
        {
            let mut i: usize = pos + 15;
            while i < header.len() && (header[i] == b' ' || header[i] == b'\t') { i += 1; }
            let mut j: usize = i;
            while j < header.len() && (header[j] as char).is_ascii_digit() { j += 1; }
            String::from_utf8_lossy(&header[i..j]).to_string()
        }
        
        None => String::new(),
    };

    let len: usize = if cl_str.is_empty() { 0 } else { cl_str.parse().unwrap_or(0) };

    let mut body: Vec<u8> = Vec::with_capacity(len);
    if buf.len() > he {
        let rem = &buf[he..];
        body.extend_from_slice(rem);
    }

    while body.len() < len {
        let to_read = std::cmp::min(8192, len - body.len());
        let mut tmpb = vec![0u8; to_read];
        let n = stream.read(&mut tmpb)?;
        if n == 0 { return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof)); }
        body.extend_from_slice(&tmpb[..n]);
    }

    Ok(body)
}
