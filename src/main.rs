use threadpool::ThreadPool;
use std::net::{TcpStream, SocketAddr};
use std::io::Read;
use std::sync::mpsc::channel;
use std::time::Duration;
use clap::Parser;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    target: String,
    #[arg(short, long, default_value_t = 1024)]
    max_ports: u16,
    #[arg(short, long, default_value = None)]
    log_file: Option<String>,
}

fn is_open(ip: &str, port: u16) -> bool {
    let addr = format!("{}:{}", ip, port);
    let socket: SocketAddr = addr.parse().unwrap();
    let timeout = Duration::from_millis(200);
    TcpStream::connect_timeout(&socket, timeout).is_ok()
}

fn grab_banner(ip: &str, port: u16) -> Vec<String> {
    let addr = format!("{}:{}", ip, port);
    let mut banners = Vec::new();
    match TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(2)) {
        Ok(mut stream) => {
            let mut buffer = [0; 1024];
            if let Ok(bytes_read) = stream.read(&mut buffer) {
                if bytes_read > 0 {
                    let banner = String::from_utf8_lossy(&buffer[..bytes_read]);
                    println!("{}:{} => {}", ip, port, banner.trim());
                    banners.push(format!("{}:{} -> {}", ip, port, banner.trim()));
                }
            }
        }
        Err(_) => println!("{}:{} not responding", ip, port),
    }
    banners
}

fn main() {
    let args = Args::parse();
    let ip = args.target;
    let max_ports = args.max_ports;
    let log_file = args.log_file;
    let pool = ThreadPool::new(100);
    let (tx, rx) = channel();
    for port in 1..max_ports {
        let tx = tx.clone();
        let ip = ip.to_string();
        pool.execute(move || {
            if is_open(&ip, port) {
                tx.send(port).unwrap();
            }
        });
    }
    drop(tx);
    for port in rx {
        let banners = grab_banner(&ip, port);
        if let Some(ref file) = log_file {
            let mut f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(file)
                .unwrap();
            for banner in banners {
                writeln!(f, "{}", banner).unwrap();
            }
        }
        println!("Port {} is open", port);
    }

}
