extern crate rstzfile;
use std::env;
use rstzfile::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let requested_timezone = &args[1];
    parse(&requested_timezone);
}

fn parse(requested_timezone: &str) {
    let buffer = match Tzfile::read(&requested_timezone) {
        Ok(b) => b,
        Err(e) => { println!("{}",e) ; return }
    };
    let header = Tzfile::parse_header(&buffer);

    let timezone = match header {
        Ok(h) => h.parse(&buffer),
        Err(e) => { println!("{}",e) ; return }
    };

    println!("{:?}", timezone);
}
