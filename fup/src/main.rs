mod net;
use net::{write_packet, read_packet, connect_tls, TlsStream};

use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;

const PORT: u16 = 1234;

#[derive(Parser, Debug)]
struct Args
{
    #[arg(long)]
    address: String,

    #[arg(long)]
    node: String,
}

fn should_skip(path: &Path) -> bool
{
    for component in path.components()
    {
        if component.as_os_str() == "runtime"
        {
            return true;
        }
    }

    false
}

fn collect_files(
    base: &Path,
    current: &Path,
    files: &mut Vec<(PathBuf, PathBuf)>
)
{
    for entry in fs::read_dir(current).unwrap()
    {
        let entry: fs::DirEntry = entry.unwrap();
        let path: PathBuf = entry.path();

        if should_skip(&path)
        {
            continue;
        }

        if path.is_dir()
        {
            collect_files(base, &path, files);
        }
        else
        {
            let relative: PathBuf =
                path.strip_prefix(base).unwrap().to_path_buf();

            files.push((path, relative));
        }
    }
}

fn send_file(
    stream: &mut TlsStream,
    path: &Path,
    relative: &Path
) -> std::io::Result<()>
{
    let data: Vec<u8> = fs::read(path)?;

    let mut packet: Vec<u8> = Vec::new();

    packet.extend_from_slice(
        format!("FILE {}\n", relative.display()).as_bytes()
    );

    packet.extend_from_slice(&data);

    write_packet(stream, &packet)?;

    Ok(())
}

fn main() -> std::io::Result<()>
{
    let args: Args = Args::parse();

    let socket: String =
        format!("{}:{}", args.address, PORT);

    // Establish TLS connection to the server
    let mut stream: TlsStream = connect_tls(&socket, &args.address)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("TLS connect failed: {}", e)))?;
    println!("Connected to {}", socket);

    // Request upload mode
    println!("Sending upload request");
    write_packet(&mut stream, b"upload")?;

    let response = read_packet(&mut stream)?;
    println!("Received response: {:?}", response);
    if response.len() < 1 || response[0] != b'r' {
        panic!("Server rejected upload");
    }

    let node_path: PathBuf = PathBuf::from(&args.node);

    let project_name: String =
        node_path.file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

    // Start upload session
    println!("Sending UPLOAD {}", project_name);
    write_packet(
        &mut stream,
        format!("UPLOAD {}\n", project_name).as_bytes()
    )?;

    let mut files: Vec<(PathBuf, PathBuf)> = Vec::new();

    collect_files(
        &node_path,
        &node_path,
        &mut files
    );

    for (full, relative) in files
    {
        println!("Uploading {}", relative.display());

        send_file(
            &mut stream,
            &full,
            &relative
        )?;
        println!("Finished {}", relative.display());
    }

    write_packet(&mut stream, b"END\n")?;

    println!("Upload complete.");

    Ok(())
}