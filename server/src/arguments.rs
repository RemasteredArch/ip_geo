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

use std::{
    fs,
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6},
    path::Path,
};

use clap::Parser;
use serde::Deserialize;

/// Represents the command-line arguments of the program.
#[derive(Parser, Deserialize, Debug)]
#[command(about, version, long_about = None)]
pub struct Arguments {
    #[arg(short = 'f', long = "config-path")]
    #[serde(skip, default)]
    pub config_path: Option<Box<Path>>,

    #[arg(short = '4', long = "ipv4")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv4_pair: Option<SocketAddrV4>,

    #[arg(long = "ipv4-path")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv4_path: Option<Box<Path>>,

    #[arg(long = "ipv4-length")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv4_len: Option<usize>,

    #[arg(long = "ipv4-comment")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv4_comment: Option<char>,

    #[arg(short = '6', long = "ipv6")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv6_pair: Option<SocketAddrV6>,

    #[arg(long = "ipv6-path")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv6_path: Option<Box<Path>>,

    #[arg(long = "ipv6-length")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv6_len: Option<usize>,

    #[arg(long = "ipv6-comment")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv6_comment: Option<char>,
}

/// For a given `Arguments` result from Clap, return `arguments` with defaults inserted.
pub fn get_config(arguments: Arguments) -> Arguments {
    let from_config = get_config_file_arguments(&arguments).and_then(|v| v.ok());
    let from_config = from_config.as_ref();

    let config_path = arguments
        .config_path
        .unwrap_or_else(get_default_config_path);

    let ipv4_pair = arguments
        .ipv4_pair
        .or_else(|| from_config.and_then(|v| v.ipv4_pair))
        .unwrap_or(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 26_000));

    let ipv4_path = arguments
        .ipv4_path
        .unwrap_or_else(|| Path::new("/usr/share/tor/geoip").into());

    let ipv4_len = arguments
        .ipv4_len
        .or_else(|| from_config.and_then(|v| v.ipv4_len))
        .unwrap_or(200_000);

    let ipv4_comment = arguments
        .ipv4_comment
        .or_else(|| from_config.and_then(|v| v.ipv4_comment))
        .unwrap_or('#');

    let ipv6_pair = arguments
        .ipv6_pair
        .or_else(|| from_config.and_then(|v| v.ipv6_pair))
        .unwrap_or(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 26_000, 0, 0));

    let ipv6_path = arguments
        .ipv6_path
        .or_else(|| from_config.and_then(|v| v.ipv6_path.clone()))
        .unwrap_or_else(|| Path::new("/usr/share/tor/geoip6").into());

    let ipv6_len = arguments
        .ipv6_len
        .or_else(|| from_config.and_then(|v| v.ipv6_len))
        .unwrap_or(60_000);

    let ipv6_comment = arguments
        .ipv6_comment
        .or_else(|| from_config.and_then(|v| v.ipv6_comment))
        .unwrap_or('#');

    Arguments {
        config_path: Some(config_path),
        ipv4_pair: Some(ipv4_pair),
        ipv4_path: Some(ipv4_path),
        ipv4_len: Some(ipv4_len),
        ipv4_comment: Some(ipv4_comment),
        ipv6_pair: Some(ipv6_pair),
        ipv6_path: Some(ipv6_path),
        ipv6_len: Some(ipv6_len),
        ipv6_comment: Some(ipv6_comment),
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
