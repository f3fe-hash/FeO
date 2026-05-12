mod net;
use net::{write_packet, read_packet, connect_tls, TlsStream};

use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

const PORT: u16 = 1234;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli
{
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Upload a node directory to the server
    Upload {
        #[arg(long)]
        address: String,

        #[arg(long)]
        node: String,
    },

    /// Start a node on the server (create/compile/run)
    Start {
        #[arg(long)]
        address: String,

        #[arg(long)]
        node: String,
    },
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
    let cli = Cli::parse();

    match cli.command {
        Commands::Upload { address, node } => {
            let socket = format!("{}:{}", address, PORT);
            let mut stream: TlsStream = connect_tls(&socket, &address)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("TLS connect failed: {}", e)))?;

            // Request upload mode
            write_packet(&mut stream, b"upload")?;

            let response = read_packet(&mut stream)?;
            if response.len() < 1 || response[0] != b'r' {
                panic!("Server rejected upload");
            }

            let node_path: PathBuf = PathBuf::from(&node);
            let project_name: String = node_path.file_name().unwrap().to_string_lossy().to_string();

            // Start upload session
            write_packet(&mut stream, format!("UPLOAD {}\n", project_name).as_bytes())?;

            let mut files: Vec<(PathBuf, PathBuf)> = Vec::new();

            collect_files(&node_path, &node_path, &mut files);

            for (full, relative) in files {
                send_file(&mut stream, &full, &relative)?;
            }

            write_packet(&mut stream, b"END\n")?;

            println!("Upload complete.");
        }

        Commands::Start { address, node } => {
            let socket = format!("{}:{}", address, PORT);
            let mut stream: TlsStream = connect_tls(&socket, &address)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("TLS connect failed: {}", e)))?;

            // Request start mode
            write_packet(&mut stream, b"start")?;

            let response = read_packet(&mut stream)?;
            if response.len() < 1 || response[0] != b'r' {
                panic!("Server rejected start request");
            }

            let node_path: PathBuf = PathBuf::from(&node);
            let project_name: String = node_path.file_name().unwrap().to_string_lossy().to_string();

            // Send START command
            write_packet(&mut stream, format!("START {}\n", project_name).as_bytes())?;

            // Read server reply (ok or err ...)
            let reply = read_packet(&mut stream)?;
            let reply_str = String::from_utf8_lossy(&reply);
            println!("Start reply: {}", reply_str);
        }
    }

    Ok(())
}