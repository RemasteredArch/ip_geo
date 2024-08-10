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
/*
( $( ($field:ident, $default:expr) ),+ ),
$( ($clone_field:ident, $default_fn:expr), )+*/
macro_rules! inject_defaults {
    (
        $arguments:expr,
        $from_config:expr,
        [ $( ($field:ident, $default:expr), )+ ],
        [ $( ($clone_field:ident, $default_fn:expr), )+ ]
    ) => {
        $(
            let $field = $arguments
                .$field
                .or_else(|| $from_config.and_then(|v| v.$field))
                .unwrap_or($default);
        )+

        $(
            let $clone_field = $arguments
                .$clone_field
                .or_else(|| $from_config.and_then(|v| v.$clone_field.clone()))
                .unwrap_or_else($default_fn);
        )+

        Arguments {
            $(
                $field: Some($field),
            )+
            $(
                $clone_field: Some($clone_field),
            )+
        }
    };
}

/// For a given `Arguments` result from Clap, return `arguments` with defaults inserted.
pub fn get_config(arguments: Arguments) -> Arguments {
    let from_config = get_config_file_arguments(&arguments).and_then(|v| v.ok());
    let from_config = from_config.as_ref();

    inject_defaults!(
        arguments,
        from_config,
        [
            (ipv4_pair, SocketAddrV4::new(Ipv4Addr::LOCALHOST, 26_000)),
            (ipv4_db_len, 200_000),
            (ipv4_db_comment, '#'),
            (
                ipv6_pair,
                SocketAddrV6::new(Ipv6Addr::LOCALHOST, 26_000, 0, 0)
            ),
            (ipv6_db_len, 60_000),
            (ipv6_db_comment, '#'),
        ],
        [
            (config_path, get_default_config_path),
            (ipv4_db_path, || Path::new("/usr/share/tor/geoip").into()),
            (ipv6_db_path, || Path::new("/usr/share/tor/geoip6").into()),
        ]
    )
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
