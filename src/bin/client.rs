use std::net::TcpListener;
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

    /// The address, and optionally port, of the target websocket server
    #[arg(short, long, default_value = "ws://127.0.0.1:1988")]
    target: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: log::LevelFilter,
}

pub fn main() {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(args.log_level)
        .init();

    let sock = TcpListener::bind(format!("{}:{}", args.host, args.port)).unwrap();

    for stream in sock.incoming().filter_map(Result::ok) {
        log::info!("Got connection from: {:?}", stream.peer_addr());
        let mut client = websocket::ClientBuilder::new(&args.target).unwrap();
        let client = client.connect_insecure().unwrap();
        log::info!("Connected to websocket");
        std::thread::spawn(move || tcp_over_https::handle(stream, client));
    }
}