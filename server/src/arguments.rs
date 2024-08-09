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

    #[arg(long = "ipv4-db-path")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv4_db_path: Option<Box<Path>>,

    #[arg(long = "ipv4-db-length")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv4_db_len: Option<usize>,

    #[arg(long = "ipv4-db-comment")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv4_db_comment: Option<char>,

    #[arg(short = '6', long = "ipv6")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv6_pair: Option<SocketAddrV6>,

    #[arg(long = "ipv6-db-path")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv6_db_path: Option<Box<Path>>,

    #[arg(long = "ipv6-db-length")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv6_db_len: Option<usize>,

    #[arg(long = "ipv6-db-comment")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ipv6_db_comment: Option<char>,
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

    let ipv4_db_path = arguments
        .ipv4_db_path
        .unwrap_or_else(|| Path::new("/usr/share/tor/geoip").into());

    let ipv4_db_len = arguments
        .ipv4_db_len
        .or_else(|| from_config.and_then(|v| v.ipv4_db_len))
        .unwrap_or(200_000);

    let ipv4_db_comment = arguments
        .ipv4_db_comment
        .or_else(|| from_config.and_then(|v| v.ipv4_db_comment))
        .unwrap_or('#');

    let ipv6_pair = arguments
        .ipv6_pair
        .or_else(|| from_config.and_then(|v| v.ipv6_pair))
        .unwrap_or(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 26_000, 0, 0));

    let ipv6_db_path = arguments
        .ipv6_db_path
        .or_else(|| from_config.and_then(|v| v.ipv6_db_path.clone()))
        .unwrap_or_else(|| Path::new("/usr/share/tor/geoip6").into());

    let ipv6_db_len = arguments
        .ipv6_db_len
        .or_else(|| from_config.and_then(|v| v.ipv6_db_len))
        .unwrap_or(60_000);

    let ipv6_db_comment = arguments
        .ipv6_db_comment
        .or_else(|| from_config.and_then(|v| v.ipv6_db_comment))
        .unwrap_or('#');

    Arguments {
        config_path: Some(config_path),
        ipv4_pair: Some(ipv4_pair),
        ipv4_db_path: Some(ipv4_db_path),
        ipv4_db_len: Some(ipv4_db_len),
        ipv4_db_comment: Some(ipv4_db_comment),
        ipv6_pair: Some(ipv6_pair),
        ipv6_db_path: Some(ipv6_db_path),
        ipv6_db_len: Some(ipv6_db_len),
        ipv6_db_comment: Some(ipv6_db_comment),
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
