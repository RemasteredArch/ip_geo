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

use std::{fmt::Display, str::FromStr};

use mediawiki::{reqwest::Url, ApiSync, MediaWikiError};
use serde_json::Value;
use url::ParseError;

/// Represents all possible error states of this module
#[derive(thiserror::Error, Debug)]
enum Error {
    #[error(transparent)]
    Url(#[from] ParseError),
    #[error("can't split url")]
    UrlSplit,
    #[error(transparent)]
    Wiki(#[from] MediaWikiError),
    #[error("iterator operation failed")]
    Iter, // Could probably be more specific
    #[error("can't map value to object")]
    InvalidObject,
    #[error("can't map value to array")]
    InvalidArray,
    #[error("map convert value to string")]
    InvalidString,
    #[error("missing results in response")]
    MissingResults,
    #[error("missing binding in value")]
    MissingBindings,
}

fn main() {
    let mut additional_countries = vec![
        Country::new_without_id("AP", "African Regional Intellectual Property Organization"),
        Country::new_without_id("CS", "Serbia and Montenegro"),
    ];
    let countries = get_country_list(&mut additional_countries);

    // dbg!(&countries);
    print_country_list_as_rust(&countries);
}

/// Formats a list of countries as valid Rust code
fn print_country_list_as_rust(countries: &[Country]) {
    print!("static COUNTRIES: [Country; {}] = [", countries.len());

    countries.iter().for_each(|c| print!("{},", c.as_rust()));

    println!("];");
}

/// Represents a country and its ISO 3166-1 alpha-2 code, alongside a Wikidata ID (if available)
#[derive(Debug)]
#[allow(dead_code)]
struct Country {
    id: Option<Box<str>>, // Ex. Q31
    id_url: Option<Url>,  // Ex. http://www.wikidata.org/entity/Q31
    country: Box<str>,    // Ex. Belgium
    code: Box<str>,       // Ex. BE
}

impl Country {
    /// Create a new country without a Wikidata ID
    fn new_without_id(code: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        Self {
            id: None,
            id_url: None,
            country: name.as_ref().into(),
            code: code.as_ref().into(),
        }
    }

    /// Creates a new country from the result of a Wikidata query
    fn new_from_query(country_result: Value) -> Result<Self, Error> {
        // Ex. http://www.wikidata.org/entity/Q31
        let url_str = get_str_value(&country_result, "country")?;
        let id_url = Some(Url::from_str(url_str)?);

        // Ex. http://www.wikidata.org/entity/Q31 -> Q31
        let id = Some(
            id_url
                .clone()
                .unwrap()
                .path_segments() // Split by /
                .ok_or(Error::UrlSplit)?
                .last() // Get last element
                .ok_or(Error::Iter)?
                .into(),
        );

        // Ex. Belgium
        let country = get_str_value(&country_result, "countryLabel")?.into();

        // Ex. BE
        let code = get_str_value(&country_result, "code")?.into();

        Ok(Self {
            id,
            id_url,
            country,
            code,
        })
    }

    /// Formats contents as a valid construction of itself
    /// ```
    /// assert_eq!(
    ///     Country::new_without_id("EX", "Example").as_rust().as_ref(),
    ///     "Country { id: None, id_url: None, country: \"Example\".into(), code: \"EX\".into() }"
    /// );
    /// ```
    fn as_rust(&self) -> Box<str> {
        /// Wraps a string in double quotes
        fn as_str<T: Display>(str: T) -> String {
            format!("\"{}\"", str)
        }

        /// Given an option of a string, wraps the string in double quotes or returns "None" (with quotes)
        fn opt_or_str<T: Display>(option: Option<T>) -> String {
            match option {
                Some(str) => as_str(str),
                None => "None".to_string(),
            }
        }

        let id = opt_or_str(self.id.clone());
        let id_url = opt_or_str(self.id_url.clone());
        let country = as_str(self.country.clone());
        let code = as_str(self.code.clone());

        format!(
            "Country {{ id: {id}, id_url: {id_url}, country: {country}.into(), code: {code}.into() }}"
        )
        .into_boxed_str()
    }
}

/// Query Wikidata for a list of countries and their ISO 3166-1 alpha-2 codes as a `Country` slice
fn get_country_list(additional_countries: &mut Vec<Country>) -> Box<[Country]> {
    let query = r#"
SELECT
    ?country      # Ex. http://www.wikidata.org/entity/Q31
    ?countryLabel # Ex. Belgium
    ?code         # Ex. BE
WHERE
{
    ?country wdt:P31 wd:Q6256;  # For every instance of (p:31) country (wq:Q6256)
        wdt:P297 ?code.         # Get its ISO 3166-1 alpha-2 code (P297)

    SERVICE wikibase:label { bd:serviceParam wikibase:language "en". } # Or "[AUTO_LANGUAGE],en"
}
# LIMIT 300 # Should only return ~180 results, so no limit necessary
"#;

    let result = wikidata_query(query).expect("The result of a Wikidata Query");

    let mut countries = Vec::with_capacity(result.len() + additional_countries.len());

    for country in result {
        countries.push(Country::new_from_query(country).unwrap());
    }

    countries.append(additional_countries);
    countries.dedup_by_key(|c| c.code.clone());

    countries.into_boxed_slice()
}

/// Get the internal string value of a given field that holds a string in a Serde JSON value
fn get_str_value<'st>(result: &'st Value, label: &str) -> Result<&'st str, Error> {
    get_value(result, label)?
        .as_str()
        .ok_or(Error::InvalidString)
}

/// Get the value of a given field in a Serde JSON value
fn get_value<'st>(result: &'st Value, label: &str) -> Result<&'st Value, Error> {
    result
        .as_object() // Validate that the JSON result is an object
        .ok_or(Error::InvalidObject)?
        .get(label) // Get a field in that object
        .ok_or(Error::MissingBindings)?
        .get("value") // Get the internal value of that field
        .ok_or(Error::MissingBindings)
}

/// Make an arbitrary Wikidata query
fn wikidata_query(query: &str) -> Result<Vec<Value>, Error> {
    Ok(
        ApiSync::new("https://www.wikidata.org/w/api.php")? // Create a query destined for Wikidata
            .sparql_query(query)? // Make the query
            .as_object() // Validate that the JSON result is an object
            .ok_or(Error::InvalidObject)?
            .to_owned()
            .get("results") // Get the actual result (the types are already known so the other field can be ignored)
            .ok_or(Error::MissingResults)?
            .get("bindings") // Get the actual values of the result
            .ok_or(Error::MissingBindings)?
            .as_array() // Validate that the JSON result is an array
            .ok_or(Error::InvalidArray)?
            .to_owned(),
    )
}
