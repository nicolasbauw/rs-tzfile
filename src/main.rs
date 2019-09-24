extern crate rstzfile;
use std::env;
use rstzfile::*;

static MAGIC: u32 = 0x545A6966;

fn main() {
    let args: Vec<String> = env::args().collect();
    let requested_timezone = &args[1];

    let buffer = Tzfile::read(&requested_timezone);
    let header = Tzfile::parse_header(&buffer);
    if header.magic == MAGIC { println!("{:?}", header.parse(&buffer)); } else { return };
}
