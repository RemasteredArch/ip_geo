// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright © 2024 RemasteredArch
// Copyright © 2024 Jaxydog
//
// This file is part of ip_geo.
//
// ip_geo is free software: you can redistribute it and/or modify it under the terms of the GNU
// Affero General Public License as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// ip_geo is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
// even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with ip_geo. If
// not, see <https://www.gnu.org/licenses/>.

use clap::Parser;
use serde::Deserialize;
use std::{
    fmt::Display,
    fs,
    net::{Ipv4Addr, Ipv6Addr},
    path::Path,
};

/// Represents the command-line arguments of the program.
#[derive(Parser, Deserialize, Debug)]
#[command(about, version, long_about = None)]
pub struct Arguments {
    #[arg(short = 'f', long = "config-path")]
    #[serde(skip, default)]
    pub config_path: Option<Box<Path>>,

    #[arg(short = '4', long = "IPv4-addr")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv4_addr: Option<Ipv4Addr>,

    #[arg(long = "IPv4-port")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv4_port: Option<u16>,

    #[arg(long = "IPv4-path")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv4_path: Option<Box<Path>>,

    #[arg(long = "IPv4-length")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv4_len: Option<usize>,

    #[arg(long = "IPv4-comment")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv4_comment: Option<char>,

    #[arg(short = '6', long = "IPv6-addr")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv6_addr: Option<Ipv6Addr>,

    #[arg(long = "IPv6-port")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv6_port: Option<u16>,

    #[arg(long = "IPv6-path")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv6_path: Option<Box<Path>>,

    #[arg(long = "IPv6-length")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv6_len: Option<usize>,

    #[arg(long = "IPv6-comment")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv6_comment: Option<char>,

    #[arg(short = 'p', long = "port")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub port: Option<u16>,
}

impl Display for Arguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Config:")?;
        writeln!(f, " * Config: {:?}", self.config_path)?;
        writeln!(f, " * IPv4 DB: {:?}", self.ipv4_path)?;
        writeln!(f, " * IPv6 DB: {:?}", self.ipv6_path)?;
        writeln!(f, " * Server port: {:?}", self.port)
    }
}

/// For a given `Arguments` result from Clap, return `arguments` with defaults inserted.
pub fn get_config(arguments: Arguments) -> Arguments {
    let from_config = get_config_file_arguments(&arguments).and_then(|v| v.ok());

    // does this need to be read from config file?
    let config = arguments
        .config_path
        .or_else(|| from_config.as_ref().and_then(|v| v.config_path.clone()))
        .unwrap_or_else(get_default_config_path);

    // let ipv4_addr = arguments.ipv4_addr.unwrap_or(Ipv4Addr::LOCALHOST);

    // let ipv4_port = arguments.ipv4_port.unwrap_or(26_000);

    let ipv4_path = arguments
        .ipv4_path
        .unwrap_or_else(|| Path::new("/usr/share/tor/geoip").into());

    let ipv4_len = arguments
        .ipv4_len
        .or_else(|| from_config.as_ref().and_then(|v| v.ipv4_len))
        .unwrap_or(200_000);

    let ipv4_comment = arguments
        .ipv4_comment
        .or_else(|| from_config.as_ref().and_then(|v| v.ipv4_comment))
        .unwrap_or('#');

    let ipv6_path = arguments
        .ipv6_path
        .or_else(|| from_config.as_ref().and_then(|v| v.ipv6_path.clone()))
        .unwrap_or_else(|| Path::new("/usr/share/tor/geoip6").into());

    let ipv6_len = arguments
        .ipv6_len
        .or_else(|| from_config.as_ref().and_then(|v| v.ipv6_len))
        .unwrap_or(60_000);

    let ipv6_comment = arguments
        .ipv6_comment
        .or_else(|| from_config.as_ref().and_then(|v| v.ipv6_comment))
        .unwrap_or('#');

    let port = arguments
        .port
        .or_else(|| from_config.as_ref().and_then(|v| v.port))
        .unwrap_or(26_000);

    Arguments {
        config_path: Some(config),
        ipv4_addr: todo!(),
        ipv4_port: todo!(),
        ipv4_path: Some(ipv4_path),
        ipv4_len: Some(ipv4_len),
        ipv4_comment: Some(ipv4_comment),
        ipv6_addr: todo!(),
        ipv6_port: todo!(),
        ipv6_path: Some(ipv6_path),
        ipv6_len: Some(ipv6_len),
        ipv6_comment: Some(ipv6_comment),
        port: Some(port),
    }
}

/// Read the config file for the program for config values.
///
/// Values from the config file override defaults, but are overridden by command-line arguments.
fn get_config_file_arguments(arguments: &Arguments) -> Option<Result<Arguments, toml::de::Error>> {
    let config_path = arguments
        .config_path
        .clone()
        .unwrap_or_else(get_default_config_path);

    let contents = fs::read_to_string(&config_path).ok()?;
    Some(toml::from_str(&contents))
}

/// Return the default location for the configuration file.
///
/// Should be overriden by the command-line argument, if provided by the user.
fn get_default_config_path() -> Box<Path> {
    dirs::config_dir()
        .expect("An OS-specific config directory")
        .join(env!("CARGO_PKG_NAME"))
        .with_extension("toml")
        .into_boxed_path()
}
