//! This low-level library reads the system timezone information files and returns a Tz struct representing the TZfile
//! fields as described in the man page (<http://man7.org/linux/man-pages/man5/tzfile.5.html>).
//! Only compatible with V1 (32 bits) format version for the moment.
//!
//! For higher level parsing, see [my high-level parsing library](https://crates.io/crates/tzparse).
//! 
//! To keep the low-level aspect of the library, since 0.5.0 chrono is an optional feature which is not enabled by default, so tzh_timecnt_data is now the raw `i32` timestamp.
//! For libtzfile to return tzh_timecnt_data as `DateTime<Utc>`, you can either use the version 0.4.0 of libtzfile, or add this in Cargo.toml:
//! ```text
//! [dependencies.libtzfile]
//! version = "0.5.1"
//! features = ["with-chrono"]
//! ```
//! Here is an example:
//!```
//! extern crate libtzfile;
//!
//! fn main() {
//!     println!("{:?}", libtzfile::parse("America/Phoenix").expect("Timezone not found"));
//! }
//!```
//!
//! which outputs (with chrono enabled):
//!```text
//! Tz { tzh_timecnt_data: [1918-03-31T09:00:00Z, 1918-10-27T08:00:00Z,
//! 1919-03-30T09:00:00Z, 1919-10-26T08:00:00Z, 1942-02-09T09:00:00Z,
//! 1944-01-01T06:01:00Z, 1944-04-01T07:01:00Z, 1944-10-01T06:01:00Z,
//! 1967-04-30T09:00:00Z, 1967-10-29T08:00:00Z],
//! tzh_timecnt_indices: [0, 1, 0, 1, 2, 1, 2, 1, 0, 1],
//! tzh_typecnt: [Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 0 },
//! Ttinfo { tt_gmtoff: -25200, tt_isdst: 0, tt_abbrind: 1 },
//! Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 2 }],
//! tz_abbr: ["MDT", "MST", "MWT"] }
//!```
//! 
//! By default, with chrono disabled, tzh_timecnt_data will be like:
//! ```text
//! tzh_timecnt_data: [9EA63A90, 9FBB0780, A0861C90, A19AE980, CB890C90, CF17DF1C, CF8FE5AC,
//! D0811A1C, FAF87510, FBE85800]
//! ```
//! It uses system TZfiles (default location on Linux and Macos /usr/share/zoneinfo). On Windows, default expected location is HOME/.zoneinfo. You can override the TZfiles default location with the TZFILES_DIR environment variable. Example for Windows:
//!
//! $env:TZFILES_DIR="C:\Users\nbauw\Dev\rs-tzfile\zoneinfo\"; cargo run
//! 
//! The tests (cargo test) are written to match [2019c version of timezone database](https://www.iana.org/time-zones).

use byteorder::{ByteOrder, BE};
#[cfg(feature = "with-chrono")]
use chrono::prelude::*;
use dirs;
use std::{env, error, fmt, fs::File, io::prelude::*, path::PathBuf, str::from_utf8};

// TZif magic four bytes
static MAGIC: u32 = 0x545A6966;
// End of first (V1) header
static V1_HEADER_END: usize = 0x2C;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TzError {
    // Invalid timezone
    InvalidTimezone,
    // Invalid file format.
    InvalidMagic,
    // Bad utf8 string
    BadUtf8String,
}

impl fmt::Display for TzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("tzfile error: ")?;
        f.write_str(match self {
            TzError::InvalidTimezone => "invalid timezone",
            TzError::InvalidMagic => "invalid TZfile",
            TzError::BadUtf8String => "bad utf8 string"
        })
    }
}

impl From<std::io::Error> for TzError {
    fn from(_e: std::io::Error) -> TzError {
        TzError::InvalidTimezone
    }
}

impl From<std::str::Utf8Error> for TzError {
    fn from(_e: std::str::Utf8Error) -> TzError {
        TzError::BadUtf8String
    }
}

impl error::Error for TzError {}

impl From<TzError> for std::io::Error {
    fn from(e: TzError) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    }
}

#[cfg(not(feature = "with-chrono"))]
#[derive(Debug)]
pub struct Tz {
    pub tzh_timecnt_data: Vec<i32>,
    pub tzh_timecnt_indices: Vec<u8>,
    pub tzh_typecnt: Vec<Ttinfo>,
    pub tz_abbr: Vec<String>,
}

#[cfg(feature = "with-chrono")]
#[derive(Debug)]
pub struct Tz {
    pub tzh_timecnt_data: Vec<DateTime<Utc>>,
    pub tzh_timecnt_indices: Vec<u8>,
    pub tzh_typecnt: Vec<Ttinfo>,
    pub tz_abbr: Vec<String>,
}

#[derive(Debug)]
pub struct Ttinfo {
    pub tt_gmtoff: isize,
    pub tt_isdst: u8,
    pub tt_abbrind: u8,
}

#[derive(Debug, PartialEq)]
struct Header {
    /* For future use
    magic: u32,
    version: u8,
    tzh_ttisgmtcnt: usize,
    tzh_ttisstdcnt: usize,
    */
    tzh_leapcnt: usize,
    tzh_timecnt: usize,
    tzh_typecnt: usize,
    tzh_charcnt: usize,
}

pub fn parse(tz: &str) -> Result<Tz, TzError> {
    // Parses TZfile header
    let header = parse_header(tz)?;
    // Parses data
    parse_data(header, tz)
}

fn parse_header(tz: &str) -> Result<Header, TzError> {
    let buffer = read(tz)?;
    let magic = BE::read_u32(&buffer[0x00..=0x03]);
    if magic != MAGIC {
        return Err(TzError::InvalidMagic);
    }
    Ok(Header {
        /* For future use
        magic: magic,
        version: buffer[4],
        tzh_ttisgmtcnt: BE::read_i32(&buffer[0x14..=0x17]) as usize,
        tzh_ttisstdcnt: BE::read_i32(&buffer[0x18..=0x1B]) as usize,
        */
        tzh_leapcnt: BE::read_i32(&buffer[0x1C..=0x1F]) as usize,
        tzh_timecnt: BE::read_i32(&buffer[0x20..=0x23]) as usize,
        tzh_typecnt: BE::read_i32(&buffer[0x24..=0x27]) as usize,
        tzh_charcnt: BE::read_i32(&buffer[0x28..=0x2b]) as usize,
    })
}

fn parse_data(header: Header, tz: &str) -> Result<Tz, TzError> {
    let buffer = read(tz)?;
    // Calculates fields lengths and indexes (Version 1 format)
    let tzh_timecnt_len: usize = header.tzh_timecnt * 5;
    let tzh_typecnt_len: usize = header.tzh_typecnt * 6;
    let tzh_leapcnt_len: usize = header.tzh_leapcnt * 8;
    let tzh_charcnt_len: usize = header.tzh_charcnt;
    let tzh_timecnt_end: usize = V1_HEADER_END + tzh_timecnt_len;
    let tzh_typecnt_end: usize = tzh_timecnt_end + tzh_typecnt_len;
    let tzh_leapcnt_end: usize = tzh_typecnt_end + tzh_leapcnt_len;
    let tzh_charcnt_end: usize = tzh_leapcnt_end + tzh_charcnt_len;

    // Extracting data fields
    #[cfg(not(feature = "with-chrono"))]
    let tzh_timecnt_data: Vec<i32> = buffer
        [V1_HEADER_END..V1_HEADER_END + header.tzh_timecnt * 4]
        .chunks_exact(4)
        .map(|tt| BE::read_i32(tt))
        .collect();

    #[cfg(feature = "with-chrono")]
    let tzh_timecnt_data: Vec<DateTime<Utc>> = buffer
        [V1_HEADER_END..V1_HEADER_END + header.tzh_timecnt * 4]
        .chunks_exact(4)
        .map(|tt| Utc.timestamp(BE::read_i32(tt).into(), 0))
        .collect();

    let tzh_timecnt_indices: &[u8] =
        &buffer[V1_HEADER_END + header.tzh_timecnt * 4..tzh_timecnt_end];

    let tzh_typecnt: Vec<Ttinfo> = buffer[tzh_timecnt_end..tzh_typecnt_end]
        .chunks_exact(6)
        .map(|tti| Ttinfo {
            tt_gmtoff: BE::read_i32(&tti[0..4]) as isize,
            tt_isdst: tti[4],
            tt_abbrind: tti[5] / 4,
        })
        .collect();

    let mut tz_abbr: Vec<String> = from_utf8(&buffer[tzh_leapcnt_end..tzh_charcnt_end])?
        .split("\u{0}")
        .map(|st| st.to_string())
        .collect();
    // Removes last empty string
    tz_abbr.pop().unwrap();

    Ok(Tz {
        tzh_timecnt_data: tzh_timecnt_data,
        tzh_timecnt_indices: tzh_timecnt_indices.to_vec(),
        tzh_typecnt: tzh_typecnt,
        tz_abbr: tz_abbr,
    })
}

fn read(tz: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut tz_files_root = if cfg!(windows) && env::var_os("TZFILES_DIR").is_none() {
        // Default TZ files location (windows) is HOME/.zoneinfo, can be overridden by ENV
        let mut d = dirs::home_dir().unwrap_or(PathBuf::from("C:\\Users"));
        d.push(".zoneinfo");
        d
    } else {
        // ENV overrides default directory, or defaults to /usr/share/zoneinfo (Linux / MacOS)
        let mut d = PathBuf::new();
        d.push(env::var("TZFILES_DIR").unwrap_or(format!("/usr/share/zoneinfo/")));
        d
    };
    tz_files_root.push(tz);
    let mut f = File::open(tz_files_root)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_file() {
        assert_eq!(read("America/Phoenix").is_ok(), true);
    }

    #[test]
    fn parse_hdr() {
        let amph = Header { tzh_leapcnt: 0, tzh_timecnt: 11, tzh_typecnt: 4, tzh_charcnt: 16 };
        assert_eq!(parse_header("America/Phoenix").unwrap(), amph);
    }

    #[test]
    fn parse_indices() {
        let amph: [u8; 11] = [2, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2];
        assert_eq!(parse("America/Phoenix").unwrap().tzh_timecnt_indices, amph);
    }

    #[cfg(feature = "with-chrono")]
    #[test]
    fn parse_timedata() {
        let amph: Vec<DateTime<Utc>> = vec![
            Utc.ymd(1901, 12, 13).and_hms(20, 45, 52), // 0x80000000
            Utc.ymd(1918, 3, 31).and_hms(9, 0, 0),
            Utc.ymd(1918, 10, 27).and_hms(8, 0, 0),
            Utc.ymd(1919, 3, 30).and_hms(9, 0, 0),
            Utc.ymd(1919, 10, 26).and_hms(8, 0, 0),
            Utc.ymd(1942, 2, 09).and_hms(9, 0, 0),
            Utc.ymd(1944, 1, 1).and_hms(6, 1, 0),
            Utc.ymd(1944, 4, 1).and_hms(7, 1, 0),
            Utc.ymd(1944, 10, 1).and_hms(6, 1, 0),
            Utc.ymd(1967, 4, 30).and_hms(9, 0, 0),
            Utc.ymd(1967, 10, 29).and_hms(8, 0, 0)];
        assert_eq!(parse("America/Phoenix").unwrap().tzh_timecnt_data, amph);
    }

    #[test]
    fn parse_ttgmtoff() {
        let amph: [isize; 3] = [-26898, -21600, -25200];
        let c: [isize; 3] = [parse("America/Phoenix").unwrap().tzh_typecnt[0].tt_gmtoff, parse("America/Phoenix").unwrap().tzh_typecnt[1].tt_gmtoff, parse("America/Phoenix").unwrap().tzh_typecnt[2].tt_gmtoff];
        assert_eq!(c, amph);
    }
}
