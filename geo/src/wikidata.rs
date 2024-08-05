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

use std::str::{self, FromStr};

use crate::Error;
use mediawiki::ApiSync;
use serde_json::Value;

/// Query Wikidata for a country's location based on a Wikidata ID.
pub fn query_for_coords_by_id(id: &str) -> (f64, f64) {
    fn parse_coords(point: &str) -> Option<(f64, f64)> {
        // Ex. "Point(4.668055555 50.641111111)" -> "4.668055555 50.641111111"
        let point = point.strip_prefix("Point(")?.strip_suffix(')')?;

        // Ex. "4.668055555 50.641111111" -> ["4.668055555", "50.641111111"]
        let (longitude, latitude) = point.split_once(' ')?;

        Some((
            f64::from_str(longitude).ok()?,
            f64::from_str(latitude).ok()?,
        ))
    }

    let query = format!(
        r#"
SELECT DISTINCT
  ?location  # Ex. Point(-98.5795 39.828175)
WHERE {{
  # SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }} # Or "[AUTO_LANGUAGE],en"
  
  wd:{id} wdt:P625 ?location. # Get its location
}}
"#
    );

    dbg!(id);

    let result = wikidata_query(&query).expect("the result of a Wikidata query");
    let result = result.first().expect("a value from Wikidata");

    let point = get_str_value(result, "location").expect("a `Point(f64, f64)`");
    let points = parse_coords(point).ok_or(Error::InvalidPoint).unwrap();
    dbg!(point, points);

    points
}

/// Query Wikidata for a country's location based on a two-letter code.
pub fn query_for_coords_by_code(code: &str) -> (f64, f64) {
    fn parse_coords(point: &str) -> Option<(f64, f64)> {
        // Ex. "Point(4.668055555 50.641111111)" -> "4.668055555 50.641111111"
        let point = point.strip_prefix("Point(")?.strip_suffix(')')?;

        // Ex. "4.668055555 50.641111111" -> ["4.668055555", "50.641111111"]
        let (longitude, latitude) = point.split_once(' ')?;

        Some((
            f64::from_str(longitude).ok()?,
            f64::from_str(latitude).ok()?,
        ))
    }

    let query = format!(
        r#"
SELECT DISTINCT
  ?item      # Ex. http://www.wikidata.org/entity/Q31
# ?itemLabel # Ex. Belgium
  ?location  # Ex. Point(-98.5795 39.828175)
WHERE {{
  VALUES ?inputCode {{
    """{code}""" # Ex. BE
  }}
  
  # SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }} # Or "[AUTO_LANGUAGE],en"
  
  ?item p:P297 ?code.       # Get items with country codes
  ?code ps:P297 ?inputCode. # Match country code against `?inputCode`
  ?item wdt:P625 ?location. # Get its location
}}
"#
    );

    dbg!(code);

    let result = wikidata_query(&query).expect("the result of a Wikidata query");
    let result = result.first().expect("a value from Wikidata");

    let point = get_str_value(result, "location").expect("a `Point(f64, f64)`");
    let points = parse_coords(point).ok_or(Error::InvalidPoint).unwrap();
    dbg!(point, points);

    points
}

/// Get the internal string value of a given field that holds a string in a Serde JSON value.
fn get_str_value<'st>(result: &'st Value, label: &str) -> Result<&'st str, Error> {
    get_value(result, label)?
        .as_str()
        .ok_or(Error::InvalidString)
}

/// Get the value of a given field in a Serde JSON value.
fn get_value<'st>(result: &'st Value, label: &str) -> Result<&'st Value, Error> {
    result
        .as_object() // Validate that the JSON result is an object
        .ok_or(Error::InvalidObject)?
        .get(label) // Get a field in that object
        .ok_or(Error::MissingBindings)?
        .get("value") // Get the internal value of that field
        .ok_or(Error::MissingBindings)
}

/// Make an arbitrary Wikidata query.
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
