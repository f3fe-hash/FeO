use std::io::{Read, Write};
use std::net::TcpStream;
use std::fs;
use std::path::{Path, PathBuf};
use clap::Parser;

const port: u16 = 1234;

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Args
{
    #[arg(long)]
    address: String,

    #[arg(long)]
    node: String,
}

fn collect_files(base: &Path, current: &Path, files: &mut Vec<(PathBuf, PathBuf)>)
{
    for entry in fs::read_dir(current).unwrap()
    {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir()
        {
            collect_files(base, &path, files);
        }
        else
        {
            let relative = path.strip_prefix(base).unwrap().to_path_buf();
            files.push((path, relative));
        }
    }
}

fn send_file(stream: &mut TcpStream, path: &Path, relative: &Path) -> std::io::Result<()>
{
    let mut file: fs::File = fs::File::open(path)?;
    let size: u64 = file.metadata()?.len();

    // Send header
    let header: String = format!("FILE {} {}\n", relative.display(), size);
    stream.write_all(header.as_bytes())?;

    // Send file bytes
    let mut buffer: [u8; 4096] = [0u8; 4096];
    let mut remaining: u64 = size;

    while remaining > 0
    {
        let read = file.read(&mut buffer)?;
        stream.write_all(&buffer[..read])?;
        remaining -= read as u64;
    }

    Ok(())
}

fn main() -> std::io::Result<()>
{
    // Collect arguments
    let args = Args::parse();
    let ip = args.address;
    let node = args.node;

    // Connect to server
    let socket = format!("{}:{}", ip, port);
    let mut stream = TcpStream::connect(socket)?;

    let msg = b"upload";
    stream.write_all(msg)?;

    let mut buffer: [u8; 1] = [0; 1];
    let bytes_read: usize = stream.read(&mut buffer)?;
    if buffer[0] == b'r'
    {
        // Start upload
        stream.write_all(format!("UPLOAD {}\n", node).as_bytes())?;

        let mut files = Vec::new();
        collect_files(Path::new(&node), Path::new(&node), &mut files);

        for (full, relative) in files {
            send_file(&mut stream, &full, &relative)?;
        }

        // Finish
        stream.write_all(b"END\n")?;
    }

    Ok(())
}
