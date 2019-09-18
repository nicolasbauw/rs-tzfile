use std::io::prelude::*;
use std::fs::File;
//use std::str::from_utf8;
use byteorder::{ByteOrder, BE};

// TZif magic four bytes
static MAGIC: u32 = 0x545A6966;

#[derive(Debug, PartialEq, Eq, Clone)]
struct Tzfile {
    magic: u32,
    version: u8,
    tzh_ttisgmtcnt: i32,
    tzh_ttisstdcnt: i32,
    tzh_leapcnt: i32,
    tzh_timecnt: i32,
    tzh_typecnt: i32,
    tzh_charcnt: i32,
}

fn main() {
    read_tzdata();
}

fn read_tzdata() {
    let mut f = File::open("/Users/nicolasb/Dev/tz/usr/share/zoneinfo/America/Phoenix").unwrap();
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer).unwrap();
    //let magic=&buffer[0..=3];
    //println!("{:?}", from_utf8(magic));
    let header = Tzfile {
        magic: BE::read_u32(&buffer[0x00..=0x03]),
        version: buffer[4],
        tzh_ttisgmtcnt: BE::read_i32(&buffer[0x14..=0x17]),
        tzh_ttisstdcnt: BE::read_i32(&buffer[0x18..=0x1B]),
        tzh_leapcnt: BE::read_i32(&buffer[0x1C..=0x1F]),
        tzh_timecnt: BE::read_i32(&buffer[0x20..=0x23]),
        tzh_typecnt: BE::read_i32(&buffer[0x24..=0x27]),
        tzh_charcnt: BE::read_i32(&buffer[0x28..=0x2b]),
    };
    println!("{:?}", header.magic == MAGIC);
    println!("{:?}", header);
}
