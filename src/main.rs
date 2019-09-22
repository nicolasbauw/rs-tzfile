use std::io::prelude::*;
use std::fs::File;
use std::env;
use std::path::Path;
use std::str::from_utf8;
use byteorder::{ByteOrder, BE};
use chrono::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let requested_timezone = &args[1];

    let buffer = Tzfile::read(&requested_timezone);
    let header = Tzfile::header(&buffer);
    println!("Valid TZfile : {}", (header.magic == MAGIC));
    println!("{:?}", header);
    header.parse(&buffer);
}

// TZif magic four bytes
static MAGIC: u32 = 0x545A6966;
// End of first (V1) header
static V1_HEADER_END: usize = 0x2C;

#[derive(Debug, PartialEq, Eq, Clone)]
struct ttinfo {
        tt_gmtoff: i32,
        tt_isdst: u8,
        tt_abbrind: u8,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Tzfile {
    magic: u32,
    version: u8,
    tzh_ttisgmtcnt: usize,
    tzh_ttisstdcnt: usize,
    tzh_leapcnt: usize,
    tzh_timecnt: usize,
    tzh_typecnt: usize,
    tzh_charcnt: usize,
}

impl Tzfile {
    fn header(buffer: &[u8]) -> Tzfile {
        Tzfile {
            magic: BE::read_u32(&buffer[0x00..=0x03]),
            version: buffer[4],
            tzh_ttisgmtcnt: BE::read_i32(&buffer[0x14..=0x17]) as usize,
            tzh_ttisstdcnt: BE::read_i32(&buffer[0x18..=0x1B]) as usize,
            tzh_leapcnt: BE::read_i32(&buffer[0x1C..=0x1F]) as usize,
            tzh_timecnt: BE::read_i32(&buffer[0x20..=0x23]) as usize,
            tzh_typecnt: BE::read_i32(&buffer[0x24..=0x27]) as usize,
            tzh_charcnt: BE::read_i32(&buffer[0x28..=0x2b]) as usize,
        }
    }

    fn parse(&self, buffer: &[u8]) {
        // Calculates fields lengths and indexes (Version 1 format)
        let tzh_timecnt_len: usize = self.tzh_timecnt*5;
        let tzh_typecnt_len: usize = self.tzh_typecnt*6;
        let tzh_leapcnt_len: usize = self.tzh_leapcnt*4;
        let tzh_charcnt_len: usize = self.tzh_charcnt;
        let tzh_timecnt_end: usize = V1_HEADER_END + tzh_timecnt_len;
        let tzh_typecnt_end: usize = tzh_timecnt_end + tzh_typecnt_len;
        let tzh_leapcnt_end: usize = tzh_typecnt_end + tzh_leapcnt_len;
        let tzh_charcnt_end: usize = tzh_leapcnt_end + tzh_charcnt_len;
        println!("tzh_timecnt_len (dec): {:?}       tzh_timecnt_end (hex): {:x?}", tzh_timecnt_len, tzh_timecnt_end);
        println!("tzh_typecnt_len (dec): {:?}       tzh_typecnt_end (hex): {:x?}", tzh_typecnt_len, tzh_typecnt_end);
        println!("tzh_leapcnt_len (dec): {:?}       tzh_leapcnt_end (hex): {:x?}", tzh_leapcnt_len, tzh_leapcnt_end);
        println!("tzh_charcnt_len (dec): {:?}       tzh_charcnt_end (hex): {:x?}", tzh_charcnt_len, tzh_charcnt_end);

        let tzh_timecnt_data: Vec<&[u8]> = buffer[V1_HEADER_END..V1_HEADER_END+self.tzh_timecnt*4]
            .chunks_exact(4)
            .collect();
            println!("tzh_timecnt : {:x?}", tzh_timecnt_data);

        let tzh_timecnt_indices: Vec<&[u8]> = buffer[V1_HEADER_END+self.tzh_timecnt*4..tzh_timecnt_end]
            .chunks(1)
            .collect();
            println!("tzh_timecnt : {:x?}", tzh_timecnt_indices);

        let tzh_typecnt: Vec<ttinfo> = buffer[tzh_timecnt_end..tzh_typecnt_end]
            .chunks_exact(6)
            .map(|tti| {
                ttinfo {
                    tt_gmtoff: BE::read_i32(&tti[0..4]),
                    tt_isdst: tti[4],
                    tt_abbrind: tti[5],
                }
            })
            .collect();
            println!("tzh_timecnt : {:?}", tzh_typecnt);

        let names = from_utf8(&buffer[tzh_leapcnt_end..tzh_charcnt_end]).unwrap();
        /*let names: Vec<&str> = buffer[tzh_leapcnt_end..tzh_charcnt_end]
            .chunks_exact(4)
            .map(|char| { from_utf8(char).unwrap() })
            .collect();*/
        println!("Timezone names : {}", names);

        let seconds=BE::read_i32(&buffer[0x36..0x40]);
        //let offset = FixedOffset::east_opt(seconds);
        println!("UTC offset : {:?}", seconds);
        //println!("{:?}", offset);
    }

    fn read(tz: &str) -> Vec<u8> {
    let mut tz_files_root = env::var("DATA_ROOT").unwrap_or(format!("/Users/nicolasb/Dev/tz/usr/share/zoneinfo/"));
    //let mut tz_files_root: String = String::from("/Users/nicolasb/Dev/tz/usr/share/zoneinfo/");
    tz_files_root.push_str(tz);
    let path = Path::new(&tz_files_root);
    let mut f = File::open(path).unwrap();
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer).unwrap();
    buffer
    }
}