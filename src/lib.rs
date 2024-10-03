//! This library reads and parses the system timezone information files (TZ Files) provided by IANA.
//!
//! The default feature is ```std```. With ```default-features = false```, the crate is ```no_std``` and uses ```alloc::vec```. In both cases the ```new()``` method returns a Tz struct containing the TZfile
//! fields as described in the man page (<http://man7.org/linux/man-pages/man5/tzfile.5.html>).
//!
//! - with ```no_std``` the function signature is ```new(buf: Vec<u8>)``` where ```buf``` is the TZ File data
//!
//!```text
//! // no_std
//! [dependencies]
//! libtzfile = { version = "3.1.0", default-features = false }
//! ```
//! ```text
//! let tzfile = include_bytes!("/usr/share/zoneinfo/America/Phoenix").to_vec();
//! let tz = Tz::new(tzfile).unwrap();
//! ```
//!
//! - with ```std``` which is the default feature the function signature is ```new(tz: &str)``` where ```tz``` is the TZ File name
//!
//!```text
//! // std is the default
//! [dependencies]
//! libtzfile = "3.1.0"
//! ```
//!
//!```text
//! use libtzfile::Tz;
//! let tzfile: &str = "/usr/share/zoneinfo/America/Phoenix";
//! println!("{:?}", Tz::new(tzfile).unwrap());
//!```
//!
//!```text
//! Tz { tzh_timecnt_data: [-2717643600, -1633273200, -1615132800, -1601823600, -1583683200, -880210800, -820519140, -812653140, -796845540, -84380400, -68659200], tzh_timecnt_indices: [2, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2], tzh_typecnt: [Ttinfo { tt_utoff: -26898, tt_isdst: 0, tt_abbrind: 0 }, Ttinfo { tt_utoff: -21600, tt_isdst: 1, tt_abbrind: 1 }, Ttinfo { tt_utoff: -25200, tt_isdst: 0, tt_abbrind: 2 }, Ttinfo { tt_utoff: -21600, tt_isdst: 1, tt_abbrind: 3 }], tz_abbr: ["LMT", "MDT", "MST", "MWT"] }
//! ```
//!
//! For higher level parsing, you can enable the **parse** or **json** features.
//! For instance, to display 2020 DST transitions in France, you can use the transition_times method:
//!
//! ```text
//! use libtzfile::Tz;
//! let tzfile: &str = "/usr/share/zoneinfo/Europe/Paris";
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
//! use libtzfile::Tz;
//! let tzfile: &str = "/usr/share/zoneinfo/Europe/Paris";
//! println!("{:?}", Tz::new(tzfile).unwrap().zoneinfo().unwrap());
//!```
//!
//! ```text
//! Tzinfo { timezone: "Europe/Paris", utc_datetime: 2020-09-05T16:41:44.279502100Z, datetime: 2020-09-05T18:41:44.279502100+02:00, dst_from: Some(2020-03-29T01:00:00Z), dst_until: Some(2020-10-25T01:00:00Z), dst_period: true, raw_offset: 3600, dst_offset: 7200, utc_offset: +02:00, abbreviation: "CEST", week_number: 36 }
//! ```
//!
//! This more complete structure implements the Serialize trait and can be transformed to a json string via a method of the json feature (which includes methods from the parse feature):
//!```text
//! use libtzfile::{Tz, TzError};
//! let tzfile: &str = "/usr/share/zoneinfo/Europe/Paris";
//! let tz = Tz::new(tzfile)?
//!     .zoneinfo()?
//!     .to_json()?;
//! println!("{}", tz);
//!```
//!
//!```text
//! {"timezone":"Europe/Paris","utc_datetime":"2020-09-05T18:04:50.546668500Z","datetime":"2020-09-05T20:04:50.546668500+02:00","dst_from":"2020-03-29T01:00:00Z","dst_until":"2020-10-25T01:00:00Z","dst_period":true,"raw_offset":3600,"dst_offset":7200,"utc_offset":"+02:00","abbreviation":"CEST","week_number":36}
//!```
//!
//! This feature is used in my [world time API](https://crates.io/crates/world-time-api).
//!
//! The tests (`cargo test`, ```cargo test --no-default-features``` or ```cargo test --features parse|json```) are working with the [2024a timezone database](https://data.iana.org/time-zones/tz-link.html).

// Support using libtzfile without the standard library
#![cfg_attr(not(any(feature = "std", feature = "parse", feature = "json")), no_std)]

#[cfg(any(feature = "std", feature = "parse", feature = "json"))]
#[cfg(test)]
mod tests;
#[cfg(any(feature = "std", feature = "parse", feature = "json"))]
extern crate std;
#[cfg(any(feature = "std", feature = "parse", feature = "json"))]
use std::{
    error, fmt, fs::File, io::Read, str::from_utf8, string::String, string::ToString, vec::Vec,
};

#[cfg(not(any(feature = "std", feature = "parse", feature = "json")))]
#[cfg(test)]
mod tests_nostd;
#[cfg(not(any(feature = "std", feature = "parse", feature = "json")))]
extern crate alloc;
#[cfg(not(any(feature = "std", feature = "parse", feature = "json")))]
use alloc::{str::from_utf8, string::String, string::ToString, vec::Vec};

#[cfg(any(feature = "parse", feature = "json"))]
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
#[cfg(feature = "json")]
use serde::Serialize;

#[cfg(feature = "json")]
mod offset_serializer {
    use serde::Serialize;
    use std::{format, string::String};
    fn offset_to_json(t: chrono::FixedOffset) -> String {
        format!("{:?}", t)
    }

    pub fn serialize<S: serde::Serializer>(
        time: &chrono::FixedOffset,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        offset_to_json(time.clone()).serialize(serializer)
    }
}

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

#[cfg(any(feature = "std", feature = "parse", feature = "json"))]
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

#[cfg(any(feature = "std", feature = "parse", feature = "json"))]
impl From<std::io::Error> for TzError {
    fn from(_e: std::io::Error) -> TzError {
        TzError::InvalidTimezone
    }
}

#[cfg(any(feature = "std", feature = "parse", feature = "json"))]
impl From<std::num::ParseIntError> for TzError {
    fn from(_e: std::num::ParseIntError) -> TzError {
        TzError::ParseError
    }
}

#[cfg(any(feature = "std", feature = "parse", feature = "json"))]
impl From<std::str::Utf8Error> for TzError {
    fn from(_e: std::str::Utf8Error) -> TzError {
        TzError::BadUtf8String
    }
}

#[cfg(any(feature = "std", feature = "parse", feature = "json"))]
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

#[cfg(any(feature = "std", feature = "parse", feature = "json"))]
impl error::Error for TzError {}

/// This is the crate's primary structure, which contains the TZfile fields.
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

#[cfg(any(feature = "parse", feature = "json"))]
/// The TransitionTime struct (available with the parse or json features) contains one transition time.
#[derive(Debug, PartialEq)]
pub struct TransitionTime {
    /// The UTC time and date of the transition time, BEFORE new parameters apply
    pub time: DateTime<Utc>,
    /// The UPCOMING offset to UTC
    pub utc_offset: isize,
    /// Is upcoming change dst ?
    pub isdst: bool,
    /// TZ abbreviation of upcoming change
    pub abbreviation: String,
}

/// Convenient and human-readable informations about a timezone (available with the parse or json features).
/// With the json feature enabled, the Tzinfo struct implements the Serialize trait.
///
/// Some explanations about the offset fields:
/// - raw_offset : the "normal" offset to utc, in seconds
/// - dst_offset : the offset to utc during daylight saving time, in seconds
/// - utc_offset : the current offset to utc, taking into account daylight saving time or not (according to dst_from and dst_until), in +/- HH:MM
#[cfg(feature = "json")]
#[derive(Debug, Serialize)]
pub struct Tzinfo {
    /// Timezone name
    pub timezone: String,
    /// UTC time
    pub utc_datetime: DateTime<Utc>,
    /// Local time
    pub datetime: DateTime<FixedOffset>,
    /// Start of DST period
    pub dst_from: Option<DateTime<Utc>>,
    /// End of DST period
    pub dst_until: Option<DateTime<Utc>>,
    /// Are we in DST period ?
    pub dst_period: bool,
    /// Normal offset to UTC, in seconds
    pub raw_offset: isize,
    /// DST offset to UTC, in seconds
    pub dst_offset: isize,
    /// current offset to UTC, in +/-HH:MM
    #[serde(with = "offset_serializer")]
    pub utc_offset: FixedOffset,
    /// Timezone abbreviation
    pub abbreviation: String,
    /// Week number
    pub week_number: i32,
}

#[cfg(feature = "parse")]
#[derive(Debug)]
pub struct Tzinfo {
    /// Timezone name
    pub timezone: String,
    /// UTC time
    pub utc_datetime: DateTime<Utc>,
    /// Local time
    pub datetime: DateTime<FixedOffset>,
    /// Start of DST period
    pub dst_from: Option<DateTime<Utc>>,
    /// End of DST period
    pub dst_until: Option<DateTime<Utc>>,
    /// Are we in DST period ?
    pub dst_period: bool,
    /// Normal offset to UTC, in seconds
    pub raw_offset: isize,
    /// DST offset to UTC, in seconds
    pub dst_offset: isize,
    /// current offset to UTC, in +/-HH:MM
    pub utc_offset: FixedOffset,
    /// Timezone abbreviation
    pub abbreviation: String,
    /// Week number
    pub week_number: i32,
}

#[cfg(feature = "json")]
impl Tzinfo {
    /// Transforms the Tzinfo struct to a JSON string
    ///
    ///```rust
    /// # let tzfile = if cfg!(windows) { "c:\\Users\\nbauw\\Dev\\zoneinfo\\Europe\\Paris" } else { "/usr/share/zoneinfo/Europe/Paris" };
    /// use libtzfile::{Tz, TzError};
    /// let tz = Tz::new(tzfile)?
    ///     .zoneinfo()?
    ///     .to_json()?;
    /// println!("{}", tz);
    /// # Ok::<(), TzError>(())
    ///```
    ///
    ///```text
    /// {"timezone":"Europe/Paris","utc_datetime":"2020-09-05T18:04:50.546668500Z","datetime":"2020-09-05T20:04:50.546668500+02:00","dst_from":"2020-03-29T01:00:00Z","dst_until":"2020-10-25T01:00:00Z","dst_period":true,"raw_offset":3600,"dst_offset":7200,"utc_offset":"+02:00","abbreviation":"CEST","week_number":36}
    ///```
    pub fn to_json(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

impl Tz {
    #[cfg(not(any(feature = "std", feature = "parse", feature = "json")))]
    pub fn new(buf: Vec<u8>) -> Result<Tz, TzError> {
        // Parses TZfile header
        let header = Tz::parse_header(&buf)?;
        // Parses data
        Tz::parse_data(&buf, header)
    }

    #[cfg(any(feature = "std", feature = "parse", feature = "json"))]
    /// Creates a Tz struct from a TZ system file
    ///
    ///```rust
    /// use libtzfile::Tz;
    /// let tzfile: &str = "/usr/share/zoneinfo/America/Phoenix";
    /// println!("{:?}", Tz::new(tzfile).unwrap());
    ///```
    ///```text
    /// Tz { tzh_timecnt_data: [-2717643600, -1633273200, -1615132800, -1601823600, -1583683200, -880210800, -820519140, -812653140, -796845540, -84380400, -68659200], tzh_timecnt_indices: [2, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2], tzh_typecnt: [Ttinfo { tt_utoff: -26898, tt_isdst: 0, tt_abbrind: 0 }, Ttinfo { tt_utoff: -21600, tt_isdst: 1, tt_abbrind: 1 }, Ttinfo { tt_utoff: -25200, tt_isdst: 0, tt_abbrind: 2 }, Ttinfo { tt_utoff: -21600, tt_isdst: 1, tt_abbrind: 3 }], tz_abbr: ["LMT", "MDT", "MST", "MWT"] }
    ///```
    pub fn new(tz: &str) -> Result<Tz, TzError> {
        // Reads TZfile
        let buf = Tz::read(tz)?;
        // Parses TZfile header
        let header = Tz::parse_header(&buf)?;
        // Parses data
        Tz::parse_data(&buf, header, tz)
    }

    fn parse_header(buffer: &[u8]) -> Result<Header, TzError> {
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

    #[cfg(not(any(feature = "std", feature = "parse", feature = "json")))]
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
    fn parse_data(buffer: &[u8], header: Header, filename: &str) -> Result<Tz, TzError> {
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
            .map(BE::read_i64)
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

        let mut tz_abbr: Vec<String> = abbrs.split('\u{0}').map(|st| st.to_string()).collect();
        // Removes last empty char
        if tz_abbr.pop().is_none() {
            return Err(TzError::EmptyString);
        };

        // Generating zone name (ie. Europe/Paris) from requested file name
        let mut timezone = String::new();
        #[cfg(not(windows))]
        let mut tz: Vec<&str> = filename.split('/').collect();
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

        #[cfg(any(feature = "parse", feature = "json"))]
        {
            return Ok(Tz {
                tzh_timecnt_data,
                tzh_timecnt_indices: tzh_timecnt_indices.to_vec(),
                tzh_typecnt,
                tz_abbr,
                name: timezone,
            });
        }

        #[cfg(not(any(feature = "parse", feature = "json")))]
        {
            return Ok(Tz {
                tzh_timecnt_data,
                tzh_timecnt_indices: tzh_timecnt_indices.to_vec(),
                tzh_typecnt,
                tz_abbr,
            });
        }
    }

    #[cfg(any(feature = "std", feature = "parse", feature = "json"))]
    fn read(tz: &str) -> Result<Vec<u8>, std::io::Error> {
        let mut f = File::open(tz)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    #[cfg(any(feature = "parse", feature = "json"))]
    /// Returns year's transition times for a timezone.
    /// If year is Some(0), returns current year's transition times.
    /// If there's no transition time for selected year, returns the last occured transition time (zone's current parameters).
    /// If no year (None) is specified, returns all transition times recorded in the TZfile .
    ///
    /// ```rust
    /// # let tzfile = if cfg!(windows) { "c:\\Users\\nbauw\\Dev\\zoneinfo\\Europe\\Paris" } else { "/usr/share/zoneinfo/Europe/Paris" };
    /// use libtzfile::Tz;
    /// println!("{:?}", Tz::new(tzfile).unwrap().transition_times(Some(2020)).unwrap());
    /// ```
    ///
    /// ```text
    /// [TransitionTime { time: 2020-03-29T01:00:00Z, utc_offset: 7200, isdst: true, abbreviation: "CEST" }, TransitionTime { time: 2020-10-25T01:00:00Z, utc_offset: 3600, isdst: false, abbreviation: "CET" }]
    /// ```
    pub fn transition_times(&self, y: Option<i32>) -> Result<Vec<TransitionTime>, TzError> {
        let timezone = self;

        // Fix for issue #3 "Calling zoneinfo on a file without transition times panics"
        // We return a NoData error if no transition times are recorded in the TZFile.
        if timezone.tzh_timecnt_data.len() == 0 {
            return Err(TzError::NoData);
        }

        // used to store transition time indices
        let mut timechanges = Vec::new();
        let mut nearest_timechange: usize = 0;

        // Used to store parsed transition times
        let mut parsedtimechanges = Vec::new();

        // Get and store the transition time indices for requested
        if y.is_some() {
            let d = Utc::now();
            let y = y.unwrap();
            // year = 0 ? current year is requested
            let y = if y == 0 {
                d.format("%Y").to_string().parse()?
            } else {
                y
            };
            // for year comparison
            // We can use unwrap safely with Utc:
            // (from Chrono doc) unwrap() is best combined with time zone types where the mapping can never fail like Utc and FixedOffset.
            let yearbeg = Utc.with_ymd_and_hms(y, 1, 1, 0, 0, 0).unwrap().timestamp();
            let yearend = Utc
                .with_ymd_and_hms(y, 12, 31, 0, 0, 0)
                .unwrap()
                .timestamp();
            for t in 0..timezone.tzh_timecnt_data.len() {
                if timezone.tzh_timecnt_data[t] > yearbeg && timezone.tzh_timecnt_data[t] < yearend
                {
                    timechanges.push(t);
                }
                if timezone.tzh_timecnt_data[t] < yearbeg {
                    nearest_timechange = t;
                };
            }
        } else {
            // No year requested ? stores all transition times
            for t in 0..timezone.tzh_timecnt_data.len() {
                /* patch : chrono panics on an overflowing timestamp, and a 0xF800000000000000 timestamp is present in some Debian 10 TZfiles.*/
                if timezone.tzh_timecnt_data[t] != -576460752303423488 {
                    timechanges.push(t)
                };
            }
        }

        // Populating returned Vec<Tt>
        if timechanges.len() != 0 {
            for t in 0..timechanges.len() {
                let tc = TransitionTime {
                    time: Utc
                        .timestamp_opt(timezone.tzh_timecnt_data[timechanges[t]], 0)
                        .unwrap(),
                    utc_offset: timezone.tzh_typecnt
                        [timezone.tzh_timecnt_indices[timechanges[t]] as usize]
                        .tt_utoff,
                    isdst: timezone.tzh_typecnt
                        [timezone.tzh_timecnt_indices[timechanges[t]] as usize]
                        .tt_isdst
                        == 1,
                    abbreviation: timezone.tz_abbr[timezone.tzh_typecnt
                        [timezone.tzh_timecnt_indices[timechanges[t]] as usize]
                        .tt_abbrind as usize]
                        .to_string(),
                };
                parsedtimechanges.push(tc);
            }
        } else {
            let tc = TransitionTime {
                time: Utc
                    .timestamp_opt(timezone.tzh_timecnt_data[nearest_timechange], 0)
                    .unwrap(),
                utc_offset: timezone.tzh_typecnt
                    [timezone.tzh_timecnt_indices[nearest_timechange] as usize]
                    .tt_utoff,
                isdst: timezone.tzh_typecnt
                    [timezone.tzh_timecnt_indices[nearest_timechange] as usize]
                    .tt_isdst
                    == 1,
                abbreviation: timezone.tz_abbr[timezone.tzh_typecnt
                    [timezone.tzh_timecnt_indices[nearest_timechange] as usize]
                    .tt_abbrind as usize]
                    .to_string(),
            };
            parsedtimechanges.push(tc);
        }
        Ok(parsedtimechanges)
    }

    #[cfg(any(feature = "parse", feature = "json"))]
    /// Returns convenient data about a timezone for current date and time.
    /// ```rust
    /// # let tzfile = if cfg!(windows) { "c:\\Users\\nbauw\\Dev\\zoneinfo\\Europe\\Paris" } else { "/usr/share/zoneinfo/Europe/Paris" };
    /// use libtzfile::Tz;
    /// println!("{:?}", Tz::new(tzfile).unwrap().zoneinfo().unwrap());
    /// ```
    ///
    /// ```text
    /// Tzinfo { timezone: "Europe/Paris", utc_datetime: 2020-09-05T16:41:44.279502100Z, datetime: 2020-09-05T18:41:44.279502100+02:00, dst_from: Some(2020-03-29T01:00:00Z), dst_until: Some(2020-10-25T01:00:00Z), dst_period: true, raw_offset: 3600, dst_offset: 7200, utc_offset: +02:00, abbreviation: "CEST", week_number: 36 }
    /// ```
    pub fn zoneinfo(&self) -> Result<Tzinfo, TzError> {
        let parsedtimechanges = match self.transition_times(Some(0)) {
            Ok(p) => p,
            Err(TzError::NoData) => Vec::new(),
            Err(e) => return Err(e),
        };
        let d = Utc::now();
        if parsedtimechanges.len() == 2 {
            // 2 times changes the same year ? DST observed
            // Are we in a dst period ? true / false
            let dst = d > parsedtimechanges[0].time && d < parsedtimechanges[1].time;
            let utc_offset = if dst == true {
                FixedOffset::east_opt(parsedtimechanges[0].utc_offset as i32).unwrap()
            } else {
                FixedOffset::east_opt(parsedtimechanges[1].utc_offset as i32).unwrap()
            };
            Ok(Tzinfo {
                timezone: (self.name).clone(),
                week_number: d
                    .with_timezone(&utc_offset)
                    .format("%V")
                    .to_string()
                    .parse()?,
                utc_datetime: d,
                datetime: d.with_timezone(&utc_offset),
                dst_from: Some(parsedtimechanges[0].time),
                dst_until: Some(parsedtimechanges[1].time),
                dst_period: dst,
                raw_offset: parsedtimechanges[1].utc_offset,
                dst_offset: parsedtimechanges[0].utc_offset,
                utc_offset: utc_offset,
                abbreviation: if dst == true {
                    parsedtimechanges[0].abbreviation.clone()
                } else {
                    parsedtimechanges[1].abbreviation.clone()
                },
            })
        } else if parsedtimechanges.len() == 1 {
            let utc_offset = FixedOffset::east_opt(parsedtimechanges[0].utc_offset as i32).unwrap();
            Ok(Tzinfo {
                timezone: (self.name).clone(),
                week_number: d
                    .with_timezone(&utc_offset)
                    .format("%V")
                    .to_string()
                    .parse()?,
                utc_datetime: d,
                datetime: d.with_timezone(&utc_offset),
                dst_from: None,
                dst_until: None,
                dst_period: false,
                raw_offset: parsedtimechanges[0].utc_offset,
                dst_offset: 0,
                utc_offset: utc_offset,
                abbreviation: parsedtimechanges[0].abbreviation.clone(),
            })
        } else if parsedtimechanges.len() == 0 {
            // Addition for TZFiles that does NOT contain any transition time
            let utc_offset = FixedOffset::east_opt(self.tzh_typecnt[0].tt_utoff as i32).unwrap();
            Ok(Tzinfo {
                timezone: (self.name).clone(),
                week_number: d
                    .with_timezone(&utc_offset)
                    .format("%V")
                    .to_string()
                    .parse()?,
                utc_datetime: d,
                datetime: d.with_timezone(&utc_offset),
                dst_from: None,
                dst_until: None,
                dst_period: false,
                raw_offset: self.tzh_typecnt[0].tt_utoff,
                dst_offset: 0,
                utc_offset: utc_offset,
                abbreviation: (self.name).clone(),
            })
        } else {
            Err(TzError::NoData)
        }
    }
}
