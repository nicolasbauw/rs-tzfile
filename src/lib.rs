//! This library reads the system timezone information files provided by IANA and returns a Tz struct representing the TZfile
//! fields as described in the man page (<http://man7.org/linux/man-pages/man5/tzfile.5.html>).
//!
//! For higher level parsing, see [my high-level parsing library](https://crates.io/crates/tzparse).
//!
//! Here is an example:
//!```text
//! fn main() {
//!     println!("{:?}", libtzfile::new("/usr/share/zoneinfo/America/Phoenix").unwrap());
//! }
//!```
//!
//!```text
//! Tz { tzh_timecnt_data: [-2717643600, -1633273200, -1615132800, -1601823600, -1583683200, -880210800, -820519140, -812653140, -796845540, -84380400, -68659200],
//! tzh_timecnt_indices: [2, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2], tzh_typecnt: [Ttinfo { tt_gmtoff: -26898, tt_isdst: 0, tt_abbrind: 0 },
//! Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 1 }, Ttinfo { tt_gmtoff: -25200, tt_isdst: 0, tt_abbrind: 2 },
//! Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 3 }], tz_abbr: ["LMT", "MDT", "MST", "MWT"] }
//!```
//!
//! The tests (cargo test) are written to match [2020a version of timezone database](https://data.iana.org/time-zones/tz-link.html).

use byteorder::{ByteOrder, BE};
#[cfg(any(feature = "parse", feature = "json"))]
use chrono::{DateTime, TimeZone, Utc};
use std::{error, fmt, fs::File, io::prelude::*, str::from_utf8};

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

impl error::Error for TzError {}

impl From<TzError> for std::io::Error {
    fn from(e: TzError) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    }
}

/// struct representing the TZfile fields
#[derive(Debug)]
pub struct Tz {
    /// transition times table
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

/// a struct containing UTC offset, daylight saving time, abbreviation index
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
/// The TransitionTime struct contains one transition time from the parsed TZfile.
#[derive(Debug, PartialEq)]
pub struct TransitionTime {
    /// The UTC time and date of the transition time, BEFORE new parameters apply
    pub time: DateTime<Utc>,
    /// The UPCOMING offset to GMT
    pub gmtoff: isize,
    /// Is upcoming change dst ?
    pub isdst: bool,
    /// TZ abbreviation of upcoming change
    pub abbreviation: String,
}

impl Tz {
    /// the tz parameter is the timezone to query, ie. "/usr/share/zoneinfo/Europe/Paris"
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
    /// If there's no transition time for selected year, returns the last occured transition time to see selected zone's applying parameters.
    /// If no year (None) is specified, returns all time changes recorded in the TZfile .
    pub fn get_tt(&self, y: Option<i32>) -> Result<Vec<TransitionTime>, TzError> {
        // low-level parse of tzfile
        let timezone = self;

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
                    gmtoff: timezone.tzh_typecnt
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
                gmtoff: timezone.tzh_typecnt
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
            name: timezone
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
    #[cfg(windows)]
    static TIMEZONE: &str = "c:\\Users\\nbauw\\Dev\\zoneinfo\\America\\Phoenix";
    #[cfg(not(windows))]
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
            v2_header_start: 147,
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
        #[cfg(windows)]
        let timezone = "c:\\Users\\nbauw\\Dev\\zoneinfo\\Europe\\Amsterdam";
        #[cfg(not(windows))]
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
    #[cfg(feature = "parse")]
    #[test]
    fn partial_timechanges() {
        let tt = vec![
            TransitionTime {
                time: Utc.ymd(2019, 3, 31).and_hms(1, 0, 0),
                gmtoff: 7200,
                isdst: true,
                abbreviation: String::from("CEST"),
            },
            TransitionTime {
                time: Utc.ymd(2019, 10, 27).and_hms(1, 0, 0),
                gmtoff: 3600,
                isdst: false,
                abbreviation: String::from("CET"),
            },
        ];
        #[cfg(not(windows))]
        let tz = Tz::new("/usr/share/zoneinfo/Europe/Paris").unwrap();
        #[cfg(windows)]
        let tz = Tz::new("c:\\Users\\nbauw\\Dev\\zoneinfo\\Europe\\Paris").unwrap();
        assert_eq!(tz.get_tt(Some(2019)).unwrap(), tt);
    }

    #[cfg(feature = "parse")]
    #[test]
    fn total_timechanges() {
        let tt = vec![
            TransitionTime {
                time: Utc.ymd(1883, 11, 18).and_hms(19, 0, 0),
                gmtoff: -25200,
                isdst: false,
                abbreviation: String::from("MST"),
            },
            TransitionTime {
                time: Utc.ymd(1918, 03, 31).and_hms(9, 0, 0),
                gmtoff: -21600,
                isdst: true,
                abbreviation: String::from("MDT"),
            },
            TransitionTime {
                time: Utc.ymd(1918, 10, 27).and_hms(8, 0, 0),
                gmtoff: -25200,
                isdst: false,
                abbreviation: String::from("MST"),
            },
            TransitionTime {
                time: Utc.ymd(1919, 03, 30).and_hms(9, 0, 0),
                gmtoff: -21600,
                isdst: true,
                abbreviation: String::from("MDT"),
            },
            TransitionTime {
                time: Utc.ymd(1919, 10, 26).and_hms(8, 0, 0),
                gmtoff: -25200,
                isdst: false,
                abbreviation: String::from("MST"),
            },
            TransitionTime {
                time: Utc.ymd(1942, 02, 09).and_hms(9, 0, 0),
                gmtoff: -21600,
                isdst: true,
                abbreviation: String::from("MWT"),
            },
            TransitionTime {
                time: Utc.ymd(1944, 01, 01).and_hms(6, 1, 0),
                gmtoff: -25200,
                isdst: false,
                abbreviation: String::from("MST"),
            },
            TransitionTime {
                time: Utc.ymd(1944, 04, 01).and_hms(7, 1, 0),
                gmtoff: -21600,
                isdst: true,
                abbreviation: String::from("MWT"),
            },
            TransitionTime {
                time: Utc.ymd(1944, 10, 01).and_hms(6, 1, 0),
                gmtoff: -25200,
                isdst: false,
                abbreviation: String::from("MST"),
            },
            TransitionTime {
                time: Utc.ymd(1967, 04, 30).and_hms(9, 0, 0),
                gmtoff: -21600,
                isdst: true,
                abbreviation: String::from("MDT"),
            },
            TransitionTime {
                time: Utc.ymd(1967, 10, 29).and_hms(8, 0, 0),
                gmtoff: -25200,
                isdst: false,
                abbreviation: String::from("MST"),
            },
        ];
        #[cfg(not(windows))]
        let tz = Tz::new("/usr/share/zoneinfo/America/Phoenix").unwrap();
        #[cfg(windows)]
        let tz = Tz::new("c:\\Users\\nbauw\\Dev\\zoneinfo\\America\\Phoenix").unwrap();
        assert_eq!(tz.get_tt(None).unwrap(), tt);
    }
}
