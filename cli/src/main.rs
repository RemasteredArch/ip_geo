use clap::Parser;
use serde::Deserialize;
use std::path::Path;

fn main() {
    let arguments = Arguments::parse();
    println!("");
}

#[derive(Parser, Deserialize)]
#[command(about, version, long_about = None)]
struct Arguments {
    #[arg(short = 'f', long = "config-file")]
    #[serde(skip, default)]
    config: Option<Box<Path>>,

    #[arg(short = '4', long = "IPv4", default_value = "/usr/share/tor/geoip")]
    ipv4: Box<Path>,

    #[arg(short = '6', long = "IPv6", default_value = "/usr/share/tor/geoip6")]
    ipv6: Box<Path>,

    #[arg(short = 's', long = "server", default_value = "false")]
    server: bool,

    #[arg(short = 'p', long = "port", default_value = "26000")]
    port: u16,
}

#[allow(dead_code)]
fn default_config_path() -> Box<Path> {
    dirs::config_dir()
        .unwrap()
        .join(env!("CARGO_PKG_NAME"))
        .into_boxed_path()
}
