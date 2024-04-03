//! This library reads the system timezone information files provided by IANA and returns a Tz struct containing the TZfile
//! fields as described in the man page (<http://man7.org/linux/man-pages/man5/tzfile.5.html>).
//!
//! For higher level parsing, you can enable the **parse** or **json** features (merged from the former [tzparse](https://crates.io/crates/tzparse) library).
//!
//! In this documentation's examples, *tzfile* is the TZfile's path, for instance "/usr/share/zoneinfo/Europe/Paris".
//!
//! Without any feature enabled, one available method : new(), which returns a Tz struct:
//!```text
//! # let tzfile = if cfg!(windows) { "c:\\Users\\nbauw\\Dev\\zoneinfo\\America\\Phoenix" } else { "/usr/share/zoneinfo/Europe/Paris" };
//! use libtzfile::Tz;
//! println!("{:?}", Tz::new(tzfile).unwrap());
//!```
//!
//!```text
//! Tz { tzh_timecnt_data: [-2717643600, -1633273200, -1615132800, -1601823600, -1583683200, -880210800, -820519140, -812653140, -796845540, -84380400, -68659200], tzh_timecnt_indices: [2, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2], tzh_typecnt: [Ttinfo { tt_utoff: -26898, tt_isdst: 0, tt_abbrind: 0 }, Ttinfo { tt_utoff: -21600, tt_isdst: 1, tt_abbrind: 1 }, Ttinfo { tt_utoff: -25200, tt_isdst: 0, tt_abbrind: 2 }, Ttinfo { tt_utoff: -21600, tt_isdst: 1, tt_abbrind: 3 }], tz_abbr: ["LMT", "MDT", "MST", "MWT"] }
//! ```
//!
//! With the parse or json features enabled, you have access to additional methods.
//! For instance, to display 2020 DST transitions in France, you can use the transition_times method:
//!
//! ```text
//! # let tzfile = if cfg!(windows) { "c:\\Users\\nbauw\\Dev\\zoneinfo\\Europe\\Paris" } else { "/usr/share/zoneinfo/Europe/Paris" };
//! use libtzfile::Tz;
//! println!("{:?}", Tz::new(tzfile).unwrap().transition_times(Some(2020)).unwrap());
//! ```
//!
//! ```text
//! [TransitionTime { time: 2020-03-29T01:00:00Z, utc_offset: 7200, isdst: true, abbreviation: "CEST" }, TransitionTime { time: 2020-10-25T01:00:00Z, utc_offset: 3600, isdst: false, abbreviation: "CET" }]
//! ```
//!
//! If you want more complete information about the timezone, you can use the zoneinfo method, which returns a more complete structure:
//!
//! ```text
//! # let tzfile = if cfg!(windows) { "c:\\Users\\nbauw\\Dev\\zoneinfo\\Europe\\Paris" } else { "/usr/share/zoneinfo/Europe/Paris" };
//! use libtzfile::Tz;
//! println!("{:?}", Tz::new(tzfile).unwrap().zoneinfo().unwrap());
//!```
//!
//! ```text
//! Tzinfo { timezone: "Europe/Paris", utc_datetime: 2020-09-05T16:41:44.279502100Z, datetime: 2020-09-05T18:41:44.279502100+02:00, dst_from: Some(2020-03-29T01:00:00Z), dst_until: Some(2020-10-25T01:00:00Z), dst_period: true, raw_offset: 3600, dst_offset: 7200, utc_offset: +02:00, abbreviation: "CEST", week_number: 36 }
//! ```
//!
//! This more complete structure implements the Serialize trait and can also be transformed to a json string via a method of the json feature (which includes methods from the parse feature):
//!```text
//! # let tzfile = if cfg!(windows) { "c:\\Users\\nbauw\\Dev\\zoneinfo\\Europe\\Paris" } else { "/usr/share/zoneinfo/Europe/Paris" };
//! use libtzfile::{Tz, TzError};
//! let tz = Tz::new(tzfile)?
//!     .zoneinfo()?
//!     .to_json()?;
//! println!("{}", tz);
//! # Ok::<(), TzError>(())
//!```
//!
//!```text
//! {"timezone":"Europe/Paris","utc_datetime":"2020-09-05T18:04:50.546668500Z","datetime":"2020-09-05T20:04:50.546668500+02:00","dst_from":"2020-03-29T01:00:00Z","dst_until":"2020-10-25T01:00:00Z","dst_period":true,"raw_offset":3600,"dst_offset":7200,"utc_offset":"+02:00","abbreviation":"CEST","week_number":36}
//!```
//!
//! This feature is used in my [world time API](https://crates.io/crates/world-time-api).
//!
//! The tests (cargo test --features json) are working with the [2022a timezone database](https://data.iana.org/time-zones/tz-link.html) (MacOS 12.4).

// Support using libtzfile without the standard library
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
#[cfg(test)]
mod tests;
#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "std")]
use std::{error, fmt, fs::File, io::Read, str::from_utf8};

#[cfg(not(feature = "std"))]
#[cfg(test)]
mod tests_nostd;
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{str::from_utf8, string::String, string::ToString, vec::Vec};

use byteorder::{ByteOrder, BE};

// TZif magic four bytes
const MAGIC: u32 = 0x545A6966;
// Header length
const HEADER_LEN: usize = 0x2C;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TzError {
    // Invalid timezone
    InvalidTimezone,
    // Invalid file format.
    InvalidMagic,
    // Bad utf8 string
    BadUtf8String,
    // Only V2 format is supported
    UnsupportedFormat,
    // No data matched the request
    NoData,
    // Parsing Error
    ParseError,
    // Empty String
    EmptyString,
    // Json conversion error
    JsonError,
}

#[cfg(feature = "std")]
impl fmt::Display for TzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TZfile error : ")?;
        f.write_str(match self {
            TzError::InvalidTimezone => "Invalid timezone",
            TzError::InvalidMagic => "Invalid TZfile",
            TzError::BadUtf8String => "Bad utf8 string",
            TzError::UnsupportedFormat => "Only V2 format is supported",
            TzError::NoData => "No data matched the request",
            TzError::ParseError => "Parsing error",
            TzError::EmptyString => "Empty string",
            TzError::JsonError => "Could not convert to json",
        })
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for TzError {
    fn from(_e: std::io::Error) -> TzError {
        TzError::InvalidTimezone
    }
}

#[cfg(feature = "std")]
impl From<std::num::ParseIntError> for TzError {
    fn from(_e: std::num::ParseIntError) -> TzError {
        TzError::ParseError
    }
}

#[cfg(feature = "std")]
impl From<std::str::Utf8Error> for TzError {
    fn from(_e: std::str::Utf8Error) -> TzError {
        TzError::BadUtf8String
    }
}

#[cfg(feature = "std")]
impl From<TzError> for std::io::Error {
    fn from(e: TzError) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    }
}

#[cfg(feature = "json")]
impl From<serde_json::error::Error> for TzError {
    fn from(_e: serde_json::error::Error) -> TzError {
        TzError::JsonError
    }
}

#[cfg(feature = "std")]
impl error::Error for TzError {}

/// This is the crate's primary structure, which contains the splitted TZfile fields and optional (via features) methods.
#[derive(Debug)]
pub struct Tz {
    /// transition times timestamps table
    pub tzh_timecnt_data: Vec<i64>,
    /// indices for the next field
    pub tzh_timecnt_indices: Vec<u8>,
    /// a struct containing UTC offset, daylight saving time, abbreviation index
    pub tzh_typecnt: Vec<Ttinfo>,
    /// abbreviations table
    pub tz_abbr: Vec<String>,
    #[cfg(any(feature = "parse", feature = "json"))]
    name: String,
}

/// This sub-structure of the Tz struct is part of the TZfile format specifications, and contains UTC offset, daylight saving time, abbreviation index.
#[derive(Debug)]
pub struct Ttinfo {
    pub tt_utoff: isize,
    pub tt_isdst: u8,
    pub tt_abbrind: u8,
}

#[derive(Debug, PartialEq)]
struct Header {
    tzh_ttisutcnt: usize,
    tzh_ttisstdcnt: usize,
    tzh_leapcnt: usize,
    tzh_timecnt: usize,
    tzh_typecnt: usize,
    tzh_charcnt: usize,
    v2_header_start: usize,
}

impl Tz {
    #[cfg(not(feature = "std"))]
    pub fn new(buf: Vec<u8>) -> Result<Tz, TzError> {
        // Parses TZfile header
        let header = Tz::parse_header(&buf)?;
        // Parses data
        Tz::parse_data(&buf, header)
    }

    #[cfg(feature = "std")]
    pub fn new(tz: &str) -> Result<Tz, TzError> {
        // Reads TZfile
        let buf = Tz::read(tz)?;
        // Parses TZfile header
        let header = Tz::parse_header(&buf)?;
        // Parses data
        Tz::parse_data(&buf, header, tz)
    }

    fn parse_header(buffer: &Vec<u8>) -> Result<Header, TzError> {
        let magic = BE::read_u32(&buffer[0x00..=0x03]);
        if magic != MAGIC {
            return Err(TzError::InvalidMagic);
        }
        if buffer[4] != 50 {
            return Err(TzError::UnsupportedFormat);
        }
        let tzh_ttisutcnt = BE::read_i32(&buffer[0x14..=0x17]) as usize;
        let tzh_ttisstdcnt = BE::read_i32(&buffer[0x18..=0x1B]) as usize;
        let tzh_leapcnt = BE::read_i32(&buffer[0x1C..=0x1F]) as usize;
        let tzh_timecnt = BE::read_i32(&buffer[0x20..=0x23]) as usize;
        let tzh_typecnt = BE::read_i32(&buffer[0x24..=0x27]) as usize;
        let tzh_charcnt = BE::read_i32(&buffer[0x28..=0x2b]) as usize;
        // V2 format data start
        let s: usize = tzh_timecnt * 5
            + tzh_typecnt * 6
            + tzh_leapcnt * 8
            + tzh_charcnt
            + tzh_ttisstdcnt
            + tzh_ttisutcnt
            + 44;
        Ok(Header {
            tzh_ttisutcnt: BE::read_i32(&buffer[s + 0x14..=s + 0x17]) as usize,
            tzh_ttisstdcnt: BE::read_i32(&buffer[s + 0x18..=s + 0x1B]) as usize,
            tzh_leapcnt: BE::read_i32(&buffer[s + 0x1C..=s + 0x1F]) as usize,
            tzh_timecnt: BE::read_i32(&buffer[s + 0x20..=s + 0x23]) as usize,
            tzh_typecnt: BE::read_i32(&buffer[s + 0x24..=s + 0x27]) as usize,
            tzh_charcnt: BE::read_i32(&buffer[s + 0x28..=s + 0x2b]) as usize,
            v2_header_start: s,
        })
    }

    #[cfg(not(feature = "std"))]
    fn parse_data(buffer: &Vec<u8>, header: Header) -> Result<Tz, TzError> {
        // Calculates fields lengths and indexes (Version 2 format)
        let tzh_timecnt_len: usize = header.tzh_timecnt * 9;
        let tzh_typecnt_len: usize = header.tzh_typecnt * 6;
        let tzh_leapcnt_len: usize = header.tzh_leapcnt * 12;
        let tzh_charcnt_len: usize = header.tzh_charcnt;
        let tzh_timecnt_end: usize = HEADER_LEN + header.v2_header_start + tzh_timecnt_len;
        let tzh_typecnt_end: usize = tzh_timecnt_end + tzh_typecnt_len;
        let tzh_leapcnt_end: usize = tzh_typecnt_end + tzh_leapcnt_len;
        let tzh_charcnt_end: usize = tzh_leapcnt_end + tzh_charcnt_len;

        // Extracting data fields
        let tzh_timecnt_data: Vec<i64> = buffer[HEADER_LEN + header.v2_header_start
            ..HEADER_LEN + header.v2_header_start + header.tzh_timecnt * 8]
            .chunks_exact(8)
            .map(|tt| BE::read_i64(tt))
            .collect();

        let tzh_timecnt_indices: &[u8] =
            &buffer[HEADER_LEN + header.v2_header_start + header.tzh_timecnt * 8..tzh_timecnt_end];

        let abbrs = from_utf8(&buffer[tzh_leapcnt_end..tzh_charcnt_end]).unwrap();

        let tzh_typecnt: Vec<Ttinfo> = buffer[tzh_timecnt_end..tzh_typecnt_end]
            .chunks_exact(6)
            .map(|tti| {
                let offset = tti[5];
                let index = abbrs
                    .chars()
                    .take(offset as usize)
                    .filter(|x| *x == '\0')
                    .count();
                Ttinfo {
                    tt_utoff: BE::read_i32(&tti[0..4]) as isize,
                    tt_isdst: tti[4],
                    tt_abbrind: index as u8,
                }
            })
            .collect();

        let mut tz_abbr: Vec<String> = abbrs.split("\u{0}").map(|st| st.to_string()).collect();
        // Removes last empty char
        if tz_abbr.pop().is_none() {
            return Err(TzError::EmptyString);
        };

        Ok(Tz {
            tzh_timecnt_data,
            tzh_timecnt_indices: tzh_timecnt_indices.to_vec(),
            tzh_typecnt,
            tz_abbr,
        })
    }

    #[cfg(feature = "std")]
    fn read(tz: &str) -> Result<Vec<u8>, std::io::Error> {
        let mut f = File::open(tz)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    #[cfg(feature = "std")]
    fn parse_data(buffer: &Vec<u8>, header: Header, filename: &str) -> Result<Tz, TzError> {
        // Calculates fields lengths and indexes (Version 2 format)
        let tzh_timecnt_len: usize = header.tzh_timecnt * 9;
        let tzh_typecnt_len: usize = header.tzh_typecnt * 6;
        let tzh_leapcnt_len: usize = header.tzh_leapcnt * 12;
        let tzh_charcnt_len: usize = header.tzh_charcnt;
        let tzh_timecnt_end: usize = HEADER_LEN + header.v2_header_start + tzh_timecnt_len;
        let tzh_typecnt_end: usize = tzh_timecnt_end + tzh_typecnt_len;
        let tzh_leapcnt_end: usize = tzh_typecnt_end + tzh_leapcnt_len;
        let tzh_charcnt_end: usize = tzh_leapcnt_end + tzh_charcnt_len;

        // Extracting data fields
        let tzh_timecnt_data: Vec<i64> = buffer[HEADER_LEN + header.v2_header_start
            ..HEADER_LEN + header.v2_header_start + header.tzh_timecnt * 8]
            .chunks_exact(8)
            .map(|tt| BE::read_i64(tt))
            .collect();

        let tzh_timecnt_indices: &[u8] =
            &buffer[HEADER_LEN + header.v2_header_start + header.tzh_timecnt * 8..tzh_timecnt_end];

        let abbrs = from_utf8(&buffer[tzh_leapcnt_end..tzh_charcnt_end])?;

        let tzh_typecnt: Vec<Ttinfo> = buffer[tzh_timecnt_end..tzh_typecnt_end]
            .chunks_exact(6)
            .map(|tti| {
                let offset = tti[5];
                let index = abbrs
                    .chars()
                    .take(offset as usize)
                    .filter(|x| *x == '\0')
                    .count();
                Ttinfo {
                    tt_utoff: BE::read_i32(&tti[0..4]) as isize,
                    tt_isdst: tti[4],
                    tt_abbrind: index as u8,
                }
            })
            .collect();

        let mut tz_abbr: Vec<String> = abbrs.split("\u{0}").map(|st| st.to_string()).collect();
        // Removes last empty char
        if tz_abbr.pop().is_none() {
            return Err(TzError::EmptyString);
        };

        // Generating zone name (ie. Europe/Paris) from requested file name
        let mut timezone = String::new();
        #[cfg(not(windows))]
        let mut tz: Vec<&str> = filename.split("/").collect();
        #[cfg(windows)]
        let mut tz: Vec<&str> = filename.split("\\").collect();
        // To prevent crash (case of requested directory separator unmatching OS separator)
        if tz.len() < 3 {
            return Err(TzError::InvalidTimezone);
        }
        for _ in 0..(tz.len()) - 2 {
            tz.remove(0);
        }
        if tz[0] != "zoneinfo" {
            timezone.push_str(tz[0]);
            timezone.push_str("/");
        }
        timezone.push_str(tz[1]);

        Ok(Tz {
            tzh_timecnt_data,
            tzh_timecnt_indices: tzh_timecnt_indices.to_vec(),
            tzh_typecnt,
            tz_abbr,
        })
    }
}
