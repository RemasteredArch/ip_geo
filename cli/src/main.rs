#![allow(dead_code)]

use clap::Parser;
use ip_geo::{parse_ipv4_file, parse_ipv6_file};

mod arguments;
use arguments::{Arguments, RunType};

fn main() {
    let arguments = arguments::get_config(Arguments::parse());

    match arguments::get_run_type(&arguments) {
        RunType::Server => launch_server(arguments),
        RunType::Ipv4 => find_ipv4(arguments),
        RunType::Ipv6 => find_ipv6(arguments),
        RunType::None => todo!("Trigger help message"),
    }
}

fn find_ipv4(arguments: Arguments) {
    let mut ipv4_map = parse_ipv4_file(
        arguments
            .ipv4_path
            .expect("A valid path to an IPv4 GeoIP database"),
        arguments
            .ipv4_len
            .expect("The number of lines in the IPv4 GeoIP database"),
    );

    let input_addr = arguments.ipv4_addr.expect("A valid IPv4 Address");
    dbg!(input_addr);

    if let Some(result) = ipv4_map.search(input_addr) {
        dbg!(result);
        println!("{}", result.long_name);
    } else {
        println!("No match!");
    }
}

fn find_ipv6(arguments: Arguments) {
    let mut ipv6_map = parse_ipv6_file(
        arguments
            .ipv6_path
            .expect("A valid path to an IPv6 GeoIP database"),
        arguments
            .ipv6_len
            .expect("The number of lines in the IPv6 GeoIP database"),
    );

    let input_addr = arguments.ipv6_addr.expect("A valid IPv6 Address");
    dbg!(input_addr);

    if let Some(result) = ipv6_map.search(input_addr) {
        dbg!(result);
        println!("{}", result.long_name);
    } else {
        println!("No match!");
    }
}

fn launch_server(arguments: Arguments) {
    todo!("Implement the server functionality");
}
