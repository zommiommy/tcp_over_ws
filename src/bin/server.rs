use websocket::server::sync::Server;
use std::net::TcpStream;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The bind ip, 0.0.0.0 to listen to any ip, 127.0.0.1 to only listen from
    /// local requests
    #[arg(short, long, default_value = "0.0.0.0")]
    host: String,

    /// The port to listen on
    #[arg(short, long, default_value_t = 1987)]
    port: u16,

    /// The ip address where to forward the traffic
    #[arg(short, long, default_value = "127.0.0.1")]
    target_ip: String,

    /// The port where to forward the traffic
    #[arg(short, long, default_value_t = 22)]
    target_port: u16,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: log::LevelFilter,
}

pub fn main() {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(args.log_level)
        .init();

    let server = Server::bind(format!("{}:{}", args.host, args.port)).unwrap();

    for connection in server.filter_map(Result::ok) {
        let client = connection.accept().unwrap();
        log::info!("Got connection from websocket: {:?}", client.peer_addr());
        let stream = TcpStream::connect(format!("{}:{}", args.target_ip, args.target_port)).unwrap();
        log::info!("Connecting to tcp: {:?}", stream.peer_addr());

        std::thread::spawn(move || tcp_over_https::handle(stream, client));
    }
}



