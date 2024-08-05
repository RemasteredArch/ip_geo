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

use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use crate::{wikidata, Error};

/// Represents a country and its ISO 3166-1 alpha-2 code, alongside a Wikidata ID (if available).
#[derive(Debug, Clone)]
pub struct Country {
    pub name: Box<str>,          // Ex. Belgium
    pub code: Box<str>,          // Ex. BE
    pub coordinates: (f64, f64), // Ex. (4.668055555, 50.641111111)
}

impl Country {
    /// Create a new `Country`.
    pub fn new(code: impl AsRef<str>, name: impl AsRef<str>, coordinates: (f64, f64)) -> Self {
        Self {
            name: name.as_ref().into(),
            code: code.as_ref().into(),
            coordinates,
        }
    }

    /// Create a new `Country` from a `CountryPair` and a Wikidata query using `CountryPair.code`.
    pub fn from_pair(pair: &CountryPair) -> Self {
        let name = pair.name.clone();
        let code = pair.code.clone();
        let coordinates = wikidata::query_for_coords_by_code(&code);

        Self {
            name,
            code,
            coordinates,
        }
    }

    /// Create a new `Country` from a `CountryPair` and a Wikidata query using `id`.
    pub fn from_pair_and_id(pair: &CountryPair, id: impl AsRef<str>) -> Self {
        let name = pair.name.clone();
        let code = pair.code.clone();
        let coordinates = wikidata::query_for_coords_by_id(id.as_ref());

        Self {
            name,
            code,
            coordinates,
        }
    }

    /// Formats contents as a valid entry of `CountryData` in a `HashMap`.
    ///
    /// Example usage:
    ///
    /// ```rust
    /// assert_eq!(
    ///     Country::new("EX", "Example", (1.0, 1.0)).as_rust_map_entry(0).as_ref(),
    ///     r#"{let ex = Country {
    ///     name: "Example".into(),
    ///     code: "EX".into(),
    ///     coordinates: (1.0, -1.0),
    /// }; (ex.code.clone(), ex)},
    /// "#
    /// ])
    /// );
    /// ```
    pub fn as_rust_map_entry(&self, indent: u8) -> Box<str> {
        fn indent_string(str: &str, indent: u8) -> Box<str> {
            if indent == 0 {
                return str.to_string().into_boxed_str();
            }

            let indent = " ".repeat(indent as usize);

            let concat = move |mut output: String, line: &str| {
                writeln!(output, "{indent}{line}").expect("string concatenation");
                output
            };

            str.lines().fold(String::new(), concat).into_boxed_str()
        }

        let (code, name, coordinates) = self.contents_as_strings();
        let code_lower = match self.code.as_ref() {
            "??" => "unknown",
            _ => &format!("c_{}", self.code.to_lowercase()),
        };

        let output = format!(
            r#"{{let {code_lower} = Country {{
    name: {name},
    code: {code},
    coordinates: {coordinates},
}}; ({code_lower}.code.clone(), {code_lower})}},"#
        );

        indent_string(&output, indent)
    }

    /// Returns self as a tuple of four Strings holding string literals: `(code, name)`
    ///
    /// Example usage:
    ///
    /// ```
    /// assert_eq!(
    ///     Country::new("EX", "Example", (1.0, 1.0)).contents_as_strings()
    ///     ("\"EX\".into()", "\"Example\".into()", "(1.0, 1.0)")
    /// );
    /// ```
    fn contents_as_strings(&self) -> (Box<str>, Box<str>, Box<str>) {
        /// Wraps a string in `"` and `.into()`.
        fn str_as_str<T: Display>(str: T) -> Box<str> {
            format!("\"{}\".into()", str).into_boxed_str()
        }

        /// Format a floats tuple into a valid Rust tuple with float literals.
        fn f_tuple_as_str(tuple: (f64, f64)) -> Box<str> {
            // Formats a float into a `String` that *will* have a decimal point
            let fmt_f = |f: f64| {
                let f = f.to_string();

                match !f.contains('.') {
                    true => format!("{}.0", f),
                    false => f,
                }
            };

            format!("({}, {})", fmt_f(tuple.0), fmt_f(tuple.1)).into_boxed_str()
        }

        let (code, name, coordinates) = self.as_tuple();

        (
            str_as_str(code),
            str_as_str(name),
            f_tuple_as_str(coordinates),
        )
    }

    /// Returns the struct's internal fields as a tuple: `(code, name, coordinates)`
    ///
    /// ```rust
    /// assert_eq!(
    ///     Country::new("EX", "Example", (1.0, 1.0)).as_tuple(),
    ///     (Box::new("EX"), Box::new("Example"), (1.0, 1.0))
    /// );
    /// ```
    fn as_tuple(&self) -> (Box<str>, Box<str>, (f64, f64)) {
        (self.code.clone(), self.name.clone(), self.coordinates)
    }
}

#[derive(Clone)]
pub struct CountryPair {
    pub name: Box<str>, // Ex. Belgium
    pub code: Box<str>, // Ex. BE
}

impl CountryPair {
    /// Create a new `CountryPair`.
    pub fn new(code: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().into(),
            code: code.as_ref().into(),
        }
    }
}

impl FromStr for CountryPair {
    type Err = Error;

    /// Parse a line into a `Country`.
    ///
    /// Expects a two letter line in this format:
    ///
    /// ```text
    /// cc country name
    /// ```
    ///
    /// Where `cc` is a two letter country code, and `country name` is an arbitrary string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (code, name) = s
            .split_once(' ')
            .ok_or(Error::InvalidCountryLine(s.into()))?;

        if code.len() != 2 {
            return Err(Error::InvalidCode(code.into()));
        }

        Ok(Self::new(code, name))
    }
}
