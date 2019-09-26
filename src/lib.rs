use byteorder::{ByteOrder, BE};
use chrono::prelude::*;
use std::{env, error, fmt, fs::File, io::prelude::*, path::Path, str::from_utf8};

// TZif magic four bytes
static MAGIC: u32 = 0x545A6966;
// End of first (V1) header
static V1_HEADER_END: usize = 0x2C;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    // Invalid file format.
    InvalidMagic,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("tzfile error: ")?;
        f.write_str(match self {
            Error::InvalidMagic => "invalid TZfile",
        })
    }
}

impl error::Error for Error {}

impl From<Error> for std::io::Error {
    fn from(e: Error) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RsTz<'a> {
    pub tzh_timecnt_data: Vec<DateTime<Utc>>,
    pub tzh_timecnt_indices: &'a [u8],
    pub tzh_typecnt: Vec<Ttinfo>,
    pub tz_abbr: Vec<&'a str>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ttinfo {
    pub tt_gmtoff: isize,
    pub tt_isdst: u8,
    pub tt_abbrind: u8,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tzfile {
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
    pub fn parse_header(buffer: &[u8]) -> Result<Tzfile, Error> {
        let magic = BE::read_u32(&buffer[0x00..=0x03]);
        if magic != MAGIC {
            return Err(Error::InvalidMagic);
        }
        Ok(Tzfile {
            magic: magic,
            version: buffer[4],
            tzh_ttisgmtcnt: BE::read_i32(&buffer[0x14..=0x17]) as usize,
            tzh_ttisstdcnt: BE::read_i32(&buffer[0x18..=0x1B]) as usize,
            tzh_leapcnt: BE::read_i32(&buffer[0x1C..=0x1F]) as usize,
            tzh_timecnt: BE::read_i32(&buffer[0x20..=0x23]) as usize,
            tzh_typecnt: BE::read_i32(&buffer[0x24..=0x27]) as usize,
            tzh_charcnt: BE::read_i32(&buffer[0x28..=0x2b]) as usize,
        })
    }

    pub fn parse<'a>(&self, buffer: &'a [u8]) -> RsTz<'a> {
        // Calculates fields lengths and indexes (Version 1 format)
        let tzh_timecnt_len: usize = self.tzh_timecnt * 5;
        let tzh_typecnt_len: usize = self.tzh_typecnt * 6;
        let tzh_leapcnt_len: usize = self.tzh_leapcnt * 4;
        let tzh_charcnt_len: usize = self.tzh_charcnt;
        let tzh_timecnt_end: usize = V1_HEADER_END + tzh_timecnt_len;
        let tzh_typecnt_end: usize = tzh_timecnt_end + tzh_typecnt_len;
        let tzh_leapcnt_end: usize = tzh_typecnt_end + tzh_leapcnt_len;
        let tzh_charcnt_end: usize = tzh_leapcnt_end + tzh_charcnt_len;

        // Extracting data fields
        let tzh_timecnt_data: Vec<DateTime<Utc>> = buffer
            [V1_HEADER_END..V1_HEADER_END + self.tzh_timecnt * 4]
            .chunks_exact(4)
            .map(|tt| Utc.timestamp(BE::read_i32(tt).into(), 0))
            .collect();

        let tzh_timecnt_indices: &[u8] =
            &buffer[V1_HEADER_END + self.tzh_timecnt * 4..tzh_timecnt_end];

        let tzh_typecnt: Vec<Ttinfo> = buffer[tzh_timecnt_end..tzh_typecnt_end]
            .chunks_exact(6)
            .map(|tti| Ttinfo {
                tt_gmtoff: BE::read_i32(&tti[0..4]) as isize,
                tt_isdst: tti[4],
                tt_abbrind: tti[5] / 4,
            })
            .collect();

        let mut tz_abbr: Vec<&str> = from_utf8(&buffer[tzh_leapcnt_end..tzh_charcnt_end])
            .unwrap()
            .split("\u{0}")
            .collect();
        // Removes last empty string
        tz_abbr.pop().unwrap();

        RsTz {
            tzh_timecnt_data: tzh_timecnt_data,
            tzh_timecnt_indices: tzh_timecnt_indices,
            tzh_typecnt: tzh_typecnt,
            tz_abbr: tz_abbr,
        }
    }

    pub fn read(tz: &str) -> Result<Vec<u8>, std::io::Error> {
        let mut tz_files_root =
            env::var("DATA_ROOT").unwrap_or(format!("/Users/nicolasb/Dev/tz/usr/share/zoneinfo/"));
        tz_files_root.push_str(tz);
        let path = Path::new(&tz_files_root);
        let mut f = File::open(path)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}
