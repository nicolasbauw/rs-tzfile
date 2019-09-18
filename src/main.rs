use std::io::prelude::*;
use std::fs::File;
//use std::str::from_utf8;
use byteorder::{ByteOrder, BE};

// TZif magic four bytes
static MAGIC: u32 = 0x545A6966;
// End of first (V1) header
static V1_HEADER_END: i32 = 0x2C;

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
    let mut f = File::open("/Users/nicolasb/Dev/tz/usr/share/zoneinfo/America/Phoenix").unwrap();
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer).unwrap();
    let header = Tzfile::header(&buffer);
    println!("Valid TZfile : {}", (header.magic == MAGIC));
    println!("{:?}", header);
    header.parse();
}

impl Tzfile {
    fn header(buffer: &[u8]) -> Tzfile {
        Tzfile {
            magic: BE::read_u32(&buffer[0x00..=0x03]),
            version: buffer[4],
            tzh_ttisgmtcnt: BE::read_i32(&buffer[0x14..=0x17]),
            tzh_ttisstdcnt: BE::read_i32(&buffer[0x18..=0x1B]),
            tzh_leapcnt: BE::read_i32(&buffer[0x1C..=0x1F]),
            tzh_timecnt: BE::read_i32(&buffer[0x20..=0x23]),
            tzh_typecnt: BE::read_i32(&buffer[0x24..=0x27]),
            tzh_charcnt: BE::read_i32(&buffer[0x28..=0x2b]),
        }
    }

    fn parse(self) {
        let tzh_timecnt_len = &self.tzh_timecnt*5;
        let tzh_typecnt_len = &self.tzh_typecnt*6;
        let tzh_leapcnt_len = &self.tzh_leapcnt*4;
        let tzh_charcnt_len = &self.tzh_charcnt;
        let tzh_timecnt_end = V1_HEADER_END + tzh_timecnt_len;
        let tzh_typecnt_end = tzh_timecnt_end + tzh_typecnt_len;
        let tzh_leapcnt_end = tzh_typecnt_end + tzh_leapcnt_len;
        let tzh_charcnt_end = tzh_leapcnt_end + tzh_charcnt_len;
        println!("tzh_timecnt_end : {:x?}", tzh_timecnt_end);
        println!("tzh_typecnt_end : {:x?}", tzh_typecnt_end);
        println!("tzh_leapcnt_end : {:x?}", tzh_leapcnt_end);
        println!("tzh_charcnt_end : {:x?}", tzh_charcnt_end);
    }
}