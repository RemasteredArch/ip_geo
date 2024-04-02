use clap::Parser;
use ip_geo::IpAddrMap;
use serde::Deserialize;
use std::{fmt::Display, fs, net::Ipv4Addr, path::Path};

fn main() {
    let arguments = get_config(Arguments::parse());

    let ipv4_map: IpAddrMap<Ipv4Addr, String> = parse_ipv4_file(arguments.ipv4.unwrap());
}

fn parse_ipv4_file(path: Box<Path>) -> IpAddrMap<Ipv4Addr, String> {
    let file = fs::File::open(&path)
        .unwrap_or_else(|_| panic!("Could not open IPv4 database at {}", path.to_string_lossy()));
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    todo!()
}

#[derive(Parser, Deserialize)]
#[command(about, version, long_about = None)]
struct Arguments {
    #[arg(short = 'f', long = "config-file")]
    #[serde(skip, default)]
    config: Option<Box<Path>>,

    #[arg(short = '4', long = "IPv4")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    ipv4: Option<Box<Path>>,

    #[arg(short = '6', long = "IPv6")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    ipv6: Option<Box<Path>>,

    #[arg(short = 's', long = "server")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    server: Option<bool>,

    #[arg(short = 'p', long = "port")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    port: Option<u16>,
}

impl Display for Arguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Config:")?;
        writeln!(f, " * Config: {:?}", self.config)?;
        writeln!(f, " * IPv4 DB: {:?}", self.ipv4)?;
        writeln!(f, " * IPv6 DB: {:?}", self.ipv6)?;
        writeln!(f, " * Start as server: {:?}", self.server)?;
        writeln!(f, " * Server port: {:?}", self.port)
    }
}

fn get_config(arguments: Arguments) -> Arguments {
    let from_config = get_config_file_arguments(&arguments).and_then(|v| v.ok());

    let config = arguments
        .config
        .or_else(|| from_config.as_ref().and_then(|v| v.config.clone()))
        .unwrap_or_else(get_default_config_path);

    let ipv4 = arguments
        .ipv4
        .or_else(|| from_config.as_ref().and_then(|v| v.ipv4.clone()))
        .unwrap_or_else(|| Path::new("/usr/share/tor/geoip").into());

    let ipv6 = arguments
        .ipv6
        .or_else(|| from_config.as_ref().and_then(|v| v.ipv6.clone()))
        .unwrap_or_else(|| Path::new("/usr/share/tor/geoip6").into());

    let server = arguments
        .server
        .or_else(|| from_config.as_ref().and_then(|v| v.server))
        .unwrap_or_default();

    let port = arguments
        .port
        .or_else(|| from_config.as_ref().and_then(|v| v.port))
        .unwrap_or(26_000);

    Arguments {
        config: Some(config),
        ipv4: Some(ipv4),
        ipv6: Some(ipv6),
        server: Some(server),
        port: Some(port),
    }
}

fn get_config_file_arguments(arguments: &Arguments) -> Option<Result<Arguments, toml::de::Error>> {
    let config_path = arguments
        .config
        .clone()
        .unwrap_or_else(get_default_config_path);

    let contents = fs::read_to_string(&config_path).ok()?;
    Some(toml::from_str(&contents))
}

fn get_default_config_path() -> Box<Path> {
    dirs::config_dir()
        .unwrap()
        .join(env!("CARGO_PKG_NAME"))
        .with_extension("toml")
        .into_boxed_path()
}
