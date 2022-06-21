//! This library reads the system timezone information files provided by IANA and returns a Tz struct containing the TZfile
//! fields as described in the man page (<http://man7.org/linux/man-pages/man5/tzfile.5.html>).
//!
//! For higher level parsing, you can enable the **parse** or **json** features (merged from the former [tzparse](https://crates.io/crates/tzparse) library).
//! 
//! In this documentation's examples, *tzfile* is the TZfile's path, for instance "/usr/share/zoneinfo/Europe/Paris".
//!
//! Without any feature enabled, one available method : new(), which returns a Tz struct:
//!```rust
//! # let tzfile = if cfg!(windows) { "c:\\Users\\nbauw\\Dev\\zoneinfo\\America\\Phoenix" } else { "/usr/share/zoneinfo/Europe/Paris" };
//! use libtzfile::Tz;
//! println!("{:?}", Tz::new(tzfile).unwrap());
//!```
//!
//!```text
//! Tz { tzh_timecnt_data: [-2717643600, -1633273200, -1615132800, -1601823600, -1583683200, -880210800, -820519140, -812653140, -796845540, -84380400, -68659200], tzh_timecnt_indices: [2, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2], tzh_typecnt: [Ttinfo { tt_gmtoff: -26898, tt_isdst: 0, tt_abbrind: 0 }, Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 1 }, Ttinfo { tt_gmtoff: -25200, tt_isdst: 0, tt_abbrind: 2 }, Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 3 }], tz_abbr: ["LMT", "MDT", "MST", "MWT"] }
//! ```
//! 
//! With the parse or json features enabled, you have access to additional methods.
//! For instance, to display 2020 DST transitions in France, you can use the transition_times method:
//! 
//! ```rust
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
//! ```rust
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
//!```rust
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

use byteorder::{ByteOrder, BE};
#[cfg(any(feature = "parse", feature = "json"))]
use chrono::{DateTime, TimeZone, Utc, FixedOffset};
use std::{error, fmt, fs::File, io::prelude::*, str::from_utf8};
#[cfg(feature = "json")]
use serde::Serialize;

#[cfg(feature = "json")]
mod offset_serializer {
    use serde::Serialize;
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
    JsonError
}

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
            TzError::JsonError => "Could not convert to json"
        })
    }
}

impl From<std::io::Error> for TzError {
    fn from(_e: std::io::Error) -> TzError {
        TzError::InvalidTimezone
    }
}

impl From<std::num::ParseIntError> for TzError {
    fn from(_e: std::num::ParseIntError) -> TzError {
        TzError::ParseError
    }
}

impl From<std::str::Utf8Error> for TzError {
    fn from(_e: std::str::Utf8Error) -> TzError {
        TzError::BadUtf8String
    }
}

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
    // Zone name
    name: String,
}

/// This sub-structure of the Tz struct is part of the TZfile format specifications, and contains UTC offset, daylight saving time, abbreviation index.
#[derive(Debug)]
pub struct Ttinfo {
    pub tt_gmtoff: isize,
    pub tt_isdst: u8,
    pub tt_abbrind: u8,
}

#[derive(Debug, PartialEq)]
struct Header {
    tzh_ttisgmtcnt: usize,
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
    /// Creates a Tz struct from a timezone file.
    ///
    /// ```rust
    /// # let tzfile = if cfg!(windows) { "c:\\Users\\nbauw\\Dev\\zoneinfo\\Europe\\Paris" } else { "/usr/share/zoneinfo/Europe/Paris" };
    /// use libtzfile::Tz;
    /// let tz = Tz::new(tzfile).unwrap();
    /// ```
    /// 
    ///```text
    /// Tz { tzh_timecnt_data: [-2717643600, -1633273200, -1615132800, -1601823600, -1583683200, -880210800, -820519140, -812653140, -796845540, -84380400, -68659200], tzh_timecnt_indices: [2, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2], tzh_typecnt: [Ttinfo { tt_gmtoff: -26898, tt_isdst: 0, tt_abbrind: 0 }, Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 1 }, Ttinfo { tt_gmtoff: -25200, tt_isdst: 0, tt_abbrind: 2 }, Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 3 }], tz_abbr: ["LMT", "MDT", "MST", "MWT"] }
    /// ```
    pub fn new(tz: &str) -> Result<Tz, TzError> {
        // Reads TZfile
        let buf = Tz::read(tz)?;
        // Parses TZfile header
        let header = Tz::parse_header(&buf)?;
        // Parses data
        Tz::parse_data(&buf, header, tz)
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
            return Err(TzError::NoData)
        }

        // used to store transition time indices
        let mut timechanges = Vec::new();
        let mut nearest_timechange: usize = 0;

        // Used to store parsed transition times
        let mut parsedtimechanges = Vec::new();

        // Get and store the transition time indices for requested year
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
            let yearbeg = Utc.ymd(y, 1, 1).and_hms(0, 0, 0).timestamp();
            let yearend = Utc.ymd(y, 12, 31).and_hms(0, 0, 0).timestamp();
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
                    time: Utc.timestamp(timezone.tzh_timecnt_data[timechanges[t]], 0),
                    utc_offset: timezone.tzh_typecnt
                        [timezone.tzh_timecnt_indices[timechanges[t]] as usize]
                        .tt_gmtoff,
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
                time: Utc.timestamp(timezone.tzh_timecnt_data[nearest_timechange], 0),
                utc_offset: timezone.tzh_typecnt
                    [timezone.tzh_timecnt_indices[nearest_timechange] as usize]
                    .tt_gmtoff,
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
            Err(e) => { return Err(e) }
        };
        let d = Utc::now();
        if parsedtimechanges.len() == 2 {
            // 2 times changes the same year ? DST observed
            // Are we in a dst period ? true / false
            let dst = d > parsedtimechanges[0].time && d < parsedtimechanges[1].time;
            let utc_offset = if dst == true {
                FixedOffset::east(parsedtimechanges[0].utc_offset as i32)
            } else {
                FixedOffset::east(parsedtimechanges[1].utc_offset as i32)
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
            let utc_offset = FixedOffset::east(parsedtimechanges[0].utc_offset as i32);
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
            let utc_offset = FixedOffset::east(self.tzh_typecnt[0].tt_gmtoff as i32);
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
                raw_offset: self.tzh_typecnt[0].tt_gmtoff,
                dst_offset: 0,
                utc_offset: utc_offset,
                abbreviation: (self.name).clone(),
            })
        } else {
            Err(TzError::NoData)
        }
    }

    fn parse_header(buffer: &Vec<u8>) -> Result<Header, TzError> {
        let magic = BE::read_u32(&buffer[0x00..=0x03]);
        if magic != MAGIC {
            return Err(TzError::InvalidMagic);
        }
        if buffer[4] != 50 {
            return Err(TzError::UnsupportedFormat);
        }
        let tzh_ttisgmtcnt = BE::read_i32(&buffer[0x14..=0x17]) as usize;
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
            + tzh_ttisgmtcnt
            + 44;
        Ok(Header {
            tzh_ttisgmtcnt: BE::read_i32(&buffer[s + 0x14..=s + 0x17]) as usize,
            tzh_ttisstdcnt: BE::read_i32(&buffer[s + 0x18..=s + 0x1B]) as usize,
            tzh_leapcnt: BE::read_i32(&buffer[s + 0x1C..=s + 0x1F]) as usize,
            tzh_timecnt: BE::read_i32(&buffer[s + 0x20..=s + 0x23]) as usize,
            tzh_typecnt: BE::read_i32(&buffer[s + 0x24..=s + 0x27]) as usize,
            tzh_charcnt: BE::read_i32(&buffer[s + 0x28..=s + 0x2b]) as usize,
            v2_header_start: s,
        })
    }

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
                    tt_gmtoff: BE::read_i32(&tti[0..4]) as isize,
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
            tzh_timecnt_data: tzh_timecnt_data,
            tzh_timecnt_indices: tzh_timecnt_indices.to_vec(),
            tzh_typecnt: tzh_typecnt,
            tz_abbr: tz_abbr,
            name: timezone,
        })
    }

    fn read(tz: &str) -> Result<Vec<u8>, std::io::Error> {
        let mut f = File::open(tz)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[cfg(target_os = "windows")]
    static TIMEZONE: &str = "c:\\Users\\nbauw\\Dev\\zoneinfo\\America\\Phoenix";
    #[cfg(target_family = "unix")]
    static TIMEZONE: &str = "/usr/share/zoneinfo/America/Phoenix";
    #[test]
    fn read_file() {
        assert_eq!(Tz::read(TIMEZONE).is_ok(), true);
    }

    #[test]
    fn parse_hdr() {
        let buf = Tz::read(TIMEZONE).unwrap();
        let amph = Header {
            tzh_ttisgmtcnt: 4,
            tzh_ttisstdcnt: 4,
            tzh_leapcnt: 0,
            tzh_timecnt: 11,
            tzh_typecnt: 4,
            tzh_charcnt: 16,
            v2_header_start: 130,
        };
        assert_eq!(Tz::parse_header(&buf).unwrap(), amph);
    }

    #[test]
    fn parse_indices() {
        let amph: [u8; 11] = [2, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2];
        assert_eq!(Tz::new(TIMEZONE).unwrap().tzh_timecnt_indices, amph);
    }

    #[test]
    fn parse_timedata() {
        let amph: Vec<i64> = vec![
            -2717643600,
            -1633273200,
            -1615132800,
            -1601823600,
            -1583683200,
            -880210800,
            -820519140,
            -812653140,
            -796845540,
            -84380400,
            -68659200,
        ];
        assert_eq!(Tz::new(TIMEZONE).unwrap().tzh_timecnt_data, amph);
    }

    #[test]
    fn parse_ttgmtoff() {
        let amph: [isize; 4] = [-26898, -21600, -25200, -21600];
        let c: Vec<isize> = Tz::new(TIMEZONE)
            .unwrap()
            .tzh_typecnt
            .iter()
            .map(|ttinfo| ttinfo.tt_gmtoff)
            .collect();
        assert_eq!(c, amph);
    }

    #[test]
    fn parse_abbr() {
        let abbr: Vec<String> = vec!["LMT", "MDT", "MST", "MWT"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        assert_eq!(Tz::new(TIMEZONE).unwrap().tz_abbr, abbr);
    }

    #[test]
    fn parse_abbr_amsterdam() {
        #[cfg(target_os = "windows")]
        let timezone = "c:\\Users\\nbauw\\Dev\\zoneinfo\\Europe\\Amsterdam";
        #[cfg(target_family = "unix")]
        let timezone = "/usr/share/zoneinfo/Europe/Amsterdam";
        let abbr: Vec<String> = vec!["LMT", "NST", "AMT", "+0020", "+0120", "CET", "CEST"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        assert_eq!(Tz::new(timezone).unwrap().tz_abbr, abbr);
        dbg!(Tz::new(timezone).unwrap());
    }

    #[test]
    fn zonename() {
        let z = "America/Phoenix";
        assert_eq!(Tz::new(TIMEZONE).unwrap().name, z);
    }

    // cargo test --features=parse
    #[cfg(any(feature = "parse", feature = "json"))]
    #[test]
    fn partial_timechanges() {
        let tt = vec![
            TransitionTime {
                time: Utc.ymd(2019, 3, 31).and_hms(1, 0, 0),
                utc_offset: 7200,
                isdst: true,
                abbreviation: String::from("CEST"),
            },
            TransitionTime {
                time: Utc.ymd(2019, 10, 27).and_hms(1, 0, 0),
                utc_offset: 3600,
                isdst: false,
                abbreviation: String::from("CET"),
            },
        ];
        #[cfg(target_family = "unix")]
        let tz = Tz::new("/usr/share/zoneinfo/Europe/Paris").unwrap();
        #[cfg(target_os = "windows")]
        let tz = Tz::new("c:\\Users\\nbauw\\Dev\\zoneinfo\\Europe\\Paris").unwrap();
        assert_eq!(tz.transition_times(Some(2019)).unwrap(), tt);
    }

    #[cfg(any(feature = "parse", feature = "json"))]
    #[test]
    fn total_timechanges() {
        let tt = vec![
            TransitionTime {
                time: Utc.ymd(1883, 11, 18).and_hms(19, 0, 0),
                utc_offset: -25200,
                isdst: false,
                abbreviation: String::from("MST"),
            },
            TransitionTime {
                time: Utc.ymd(1918, 03, 31).and_hms(9, 0, 0),
                utc_offset: -21600,
                isdst: true,
                abbreviation: String::from("MDT"),
            },
            TransitionTime {
                time: Utc.ymd(1918, 10, 27).and_hms(8, 0, 0),
                utc_offset: -25200,
                isdst: false,
                abbreviation: String::from("MST"),
            },
            TransitionTime {
                time: Utc.ymd(1919, 03, 30).and_hms(9, 0, 0),
                utc_offset: -21600,
                isdst: true,
                abbreviation: String::from("MDT"),
            },
            TransitionTime {
                time: Utc.ymd(1919, 10, 26).and_hms(8, 0, 0),
                utc_offset: -25200,
                isdst: false,
                abbreviation: String::from("MST"),
            },
            TransitionTime {
                time: Utc.ymd(1942, 02, 09).and_hms(9, 0, 0),
                utc_offset: -21600,
                isdst: true,
                abbreviation: String::from("MWT"),
            },
            TransitionTime {
                time: Utc.ymd(1944, 01, 01).and_hms(6, 1, 0),
                utc_offset: -25200,
                isdst: false,
                abbreviation: String::from("MST"),
            },
            TransitionTime {
                time: Utc.ymd(1944, 04, 01).and_hms(7, 1, 0),
                utc_offset: -21600,
                isdst: true,
                abbreviation: String::from("MWT"),
            },
            TransitionTime {
                time: Utc.ymd(1944, 10, 01).and_hms(6, 1, 0),
                utc_offset: -25200,
                isdst: false,
                abbreviation: String::from("MST"),
            },
            TransitionTime {
                time: Utc.ymd(1967, 04, 30).and_hms(9, 0, 0),
                utc_offset: -21600,
                isdst: true,
                abbreviation: String::from("MDT"),
            },
            TransitionTime {
                time: Utc.ymd(1967, 10, 29).and_hms(8, 0, 0),
                utc_offset: -25200,
                isdst: false,
                abbreviation: String::from("MST"),
            },
        ];
        let tz = Tz::new(TIMEZONE).unwrap();
        assert_eq!(tz.transition_times(None).unwrap(), tt);
    }

    // cargo test --features=json
    #[cfg(any(feature = "parse", feature = "json"))]
    #[test]
    fn zoneinfo() {
        #[cfg(target_family = "unix")]
        let tztest = Tz::new("/usr/share/zoneinfo/Europe/Paris").unwrap().zoneinfo().unwrap();
        #[cfg(target_os = "windows")]
        let tztest = (Tz::new("c:\\Users\\nbauw\\Dev\\zoneinfo\\Europe\\Paris").unwrap()).zoneinfo().unwrap();
        assert_eq!(tztest.timezone, String::from("Europe/Paris"));
        assert_eq!(tztest.raw_offset, 3600);
        assert_eq!(tztest.dst_offset, 7200);
    }

    // cargo test --features=json
    #[cfg(any(feature = "parse", feature = "json"))]
    #[test]
    fn emptytt() {
        #[cfg(target_os = "windows")]
        let timezone = "c:\\Users\\nbauw\\Dev\\zoneinfo\\EST";
        #[cfg(target_family = "unix")]
        let timezone = "/usr/share/zoneinfo/EST";
        assert_eq!(Err(crate::TzError::NoData),
        Tz::new(timezone).unwrap().transition_times(None));
    }
}
