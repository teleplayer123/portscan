use threadpool::ThreadPool;
use std::net::{TcpStream, SocketAddr};
use std::sync::mpsc::channel;
use std::time::Duration;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    target: String,
    #[arg(short, long, default_value_t = 1024)]
    max_ports: u16,
}

fn is_open(ip: &str, port: u16) -> bool {
    let addr = format!("{}:{}", ip, port);
    let socket: SocketAddr = addr.parse().unwrap();
    let timeout = Duration::from_millis(200);
    TcpStream::connect_timeout(&socket, timeout).is_ok()
}

fn main() {
    let args = Args::parse();
    let ip = args.target;
    let max_ports = args.max_ports;
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
        println!("Port {} is open", port);
    }
}
