use clap::Parser;
use std::path::Path;

fn main() {
    println!("{}", u16::MAX);
}

#[derive(Parser)]
#[command(about, version, long_about = None)]
struct Arguments {
    #[arg(short = '4', long = "IPv4", default_value = "/usr/share/tor/geoip")]
    ipv4: Box<Path>,

    #[arg(short = '6', long = "IPv6", default_value = "/usr/share/tor/geoip6")]
    ipv6: Box<Path>,

    #[arg(short = 's', long = "server", default_value = "false")]
    server: bool,

    #[arg(short = 'p', long = "port", default_value = "26000")]
    port: u16,
}
