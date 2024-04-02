#![allow(dead_code)]
use celes::Country;
use clap::Parser;
use ip_geo::{IpAddrMap, Ipv4AddrEntry, Ipv6AddrEntry};
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::{
    fmt::Display,
    fs,
    net::{Ipv4Addr, Ipv6Addr},
    path::Path,
    str::FromStr,
};

fn main() {
    let arguments = get_config(Arguments::parse());

    /*
    let ipv4_map = parse_ipv4_file(arguments.ipv4_path.unwrap(), arguments.ipv4_len.unwrap());

    for ipv4_addr in ipv4_map {
        println!("{:?}", ipv4_addr);
    }*/

    let ipv6_map = parse_ipv6_file(arguments.ipv6_path.unwrap(), arguments.ipv6_len.unwrap());

    for ipv6_addr in ipv6_map {
        println!(
            "{:39}\t{:39}\t{}",
            ipv6_addr.start(),
            ipv6_addr.end(),
            ipv6_addr.value().long_name
        );
    }
}

fn parse_ipv6_file(path: Box<Path>, len: usize) -> IpAddrMap<Ipv6Addr, Country> {
    #[derive(Deserialize, Debug)]
    struct Schema {
        #[serde(deserialize_with = "deserialize_ipv6")]
        start: Ipv6Addr,

        #[serde(deserialize_with = "deserialize_ipv6")]
        end: Ipv6Addr,

        country: Country,
    }

    let file = fs::File::open(&path)
        .unwrap_or_else(|_| panic!("Could not open IPv6 database at {}", path.to_string_lossy()));
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .comment(Some(b'#'))
        .from_reader(file);

    let mut map = IpAddrMap::new_with_capacity(len);

    for entry in reader.deserialize() {
        let data: Schema = entry.unwrap();

        let entry = Ipv6AddrEntry::new(data.start, data.end, data.country).unwrap();

        map.insert(entry);
    }

    map.cleanup();

    map
}

fn deserialize_ipv6<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Ipv6Addr, D::Error> {
    pub struct Ipv6Deserializer;

    impl<'de> Visitor<'de> for Ipv6Deserializer {
        type Value = Ipv6Addr;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "an IPv6 address")
        }

        fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            const MASK: u128 = 0xFF;

            let array = std::array::from_fn(|index| {
                let shift = (7 - index) * u16::BITS as usize;

                (v >> shift & MASK) as u16
            });

            Ok(array.into())
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ipv6Addr::from_str(v).map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
    }

    deserializer.deserialize_str(Ipv6Deserializer)
}

fn parse_ipv4_file(path: Box<Path>, len: usize) -> IpAddrMap<Ipv4Addr, Country> {
    #[derive(Deserialize, Debug)]
    struct Schema {
        #[serde(deserialize_with = "deserialize_ipv4")]
        start: Ipv4Addr,

        #[serde(deserialize_with = "deserialize_ipv4")]
        end: Ipv4Addr,

        country: Country,
    }

    let file = fs::File::open(&path)
        .unwrap_or_else(|_| panic!("Could not open IPv4 database at {}", path.to_string_lossy()));
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .comment(Some(b'#'))
        .from_reader(file);

    let mut map = IpAddrMap::new_with_capacity(len);

    for entry in reader.deserialize() {
        let data: Schema = entry.unwrap();

        let entry = Ipv4AddrEntry::new(data.start, data.end, data.country).unwrap();

        map.insert(entry);
    }

    map.cleanup();

    map
}

fn deserialize_ipv4<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Ipv4Addr, D::Error> {
    pub struct Ipv4Deserializer;

    impl<'de> Visitor<'de> for Ipv4Deserializer {
        type Value = Ipv4Addr;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "an IPv4 address")
        }

        fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            const MASK: u32 = 0xFF;

            let array = std::array::from_fn(|index| {
                let shift = (3 - index) * u8::BITS as usize;

                (v >> shift & MASK) as u8
            });

            Ok(array.into())
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ipv4Addr::from_str(v).map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
    }

    deserializer.deserialize_u32(Ipv4Deserializer)
}

#[derive(Parser, Deserialize)]
#[command(about, version, long_about = None)]
struct Arguments {
    #[arg(short = 'f', long = "config-path")]
    #[serde(skip, default)]
    config_path: Option<Box<Path>>,

    // TODO: add args to configure the comment character of the DBs (e.g. # for tor_geoip)
    #[arg(short = '4', long = "IPv4-path")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    ipv4_path: Option<Box<Path>>,

    #[arg(long = "IPv4-length")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    ipv4_len: Option<usize>,

    #[arg(short = '6', long = "IPv6-path")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    ipv6_path: Option<Box<Path>>,

    #[arg(long = "IPv6-length")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    ipv6_len: Option<usize>,

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
        writeln!(f, " * Config: {:?}", self.config_path)?;
        writeln!(f, " * IPv4 DB: {:?}", self.ipv4_path)?;
        writeln!(f, " * IPv6 DB: {:?}", self.ipv6_path)?;
        writeln!(f, " * Start as server: {:?}", self.server)?;
        writeln!(f, " * Server port: {:?}", self.port)
    }
}

fn get_config(arguments: Arguments) -> Arguments {
    let from_config = get_config_file_arguments(&arguments).and_then(|v| v.ok());

    let config = arguments
        .config_path
        .or_else(|| from_config.as_ref().and_then(|v| v.config_path.clone()))
        .unwrap_or_else(get_default_config_path);

    let ipv4_path = arguments
        .ipv4_path
        .or_else(|| from_config.as_ref().and_then(|v| v.ipv4_path.clone()))
        .unwrap_or_else(|| Path::new("/usr/share/tor/geoip").into());

    let ipv4_len = arguments
        .ipv4_len
        .or_else(|| from_config.as_ref().and_then(|v| v.ipv4_len))
        .unwrap_or(200_000);

    let ipv6_path = arguments
        .ipv6_path
        .or_else(|| from_config.as_ref().and_then(|v| v.ipv6_path.clone()))
        .unwrap_or_else(|| Path::new("/usr/share/tor/geoip6").into());

    let ipv6_len = arguments
        .ipv6_len
        .or_else(|| from_config.as_ref().and_then(|v| v.ipv6_len))
        .unwrap_or(60_000);

    let server = arguments
        .server
        .or_else(|| from_config.as_ref().and_then(|v| v.server))
        .unwrap_or_default();

    let port = arguments
        .port
        .or_else(|| from_config.as_ref().and_then(|v| v.port))
        .unwrap_or(26_000);

    Arguments {
        config_path: Some(config),
        ipv4_path: Some(ipv4_path),
        ipv4_len: Some(ipv4_len),
        ipv6_path: Some(ipv6_path),
        ipv6_len: Some(ipv6_len),
        server: Some(server),
        port: Some(port),
    }
}

fn get_config_file_arguments(arguments: &Arguments) -> Option<Result<Arguments, toml::de::Error>> {
    let config_path = arguments
        .config_path
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
