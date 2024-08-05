// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright © 2024 RemasteredArch
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

use mediawiki::MediaWikiError;

/// Represents all possible error states of this crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    StrFromUtf8(#[from] core::str::Utf8Error),

    #[error("can't parse line '{0}' into Country")]
    InvalidCountryLine(Box<str>),

    #[error("expected two letter country code, received '{0}'")]
    InvalidCode(Box<str>),

    #[error("out of bounds array access")]
    OutOfBounds,

    #[error(transparent)]
    Wiki(#[from] MediaWikiError),

    #[allow(dead_code)] // Is sometimes used for debugging
    #[error("iterator operation failed")]
    Iter, // Could probably be more specific

    #[error("can't map value to object")]
    InvalidObject,

    #[error("can't map value to array")]
    InvalidArray,

    #[error("can't convert value to string")]
    InvalidString,

    #[error("can't convert string to coordinates")]
    InvalidPoint,

    #[error("missing results in response")]
    MissingResults,

    #[error("missing binding in value")]
    MissingBindings,
}
