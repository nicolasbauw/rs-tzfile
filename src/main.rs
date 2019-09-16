use std::io::prelude::*;
use std::fs::File;
use std::mem::size_of;
use std::str::from_utf8;
use byteorder::{ByteOrder, BE};

static MAGIC: [u8; 4] = *b"TZif";

#[derive(Debug, PartialEq, Eq, Clone)]
struct Header {
    tzh_leapcnt: usize,
    tzh_timecnt: usize,
    tzh_typecnt: usize,
    tzh_charcnt: usize,
}

fn main() {
    read_tzdata();
}

fn read_tzdata() {
    let mut f = File::open("/Users/nicolasb/Dev/tz/usr/share/zoneinfo/America/Phoenix").unwrap();
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer).unwrap();
    println!("{:?}", buffer);
    println!("{:?}",Header::parse(&buffer));
    let header=Header::parse(&buffer).unwrap();
    println!("{:?}",Header::data_len::<i64>(&header));
    //println!("{}",header.parse_content(&buffer[Header::HEADER_LEN..]).unwrap());
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    /// The source bytes is too short to parse the header.
    HeaderTooShort,
    /// The source does not start with the correct magic string (`"TZif"`).
    InvalidMagic,
    /// Unsupported tzfile version. Currently we only support versions 2 and 3.
    UnsupportedVersion,
    /// The lengths of several related arrays in the file are not the same,
    /// making the file invalid.
    InconsistentTypeCount,
    /// The tzfile contains no time zone information.
    NoTypes,
    /// The time zone offset exceeds ±86400s (1 day).
    OffsetOverflow,
    /// Some time zone abbreviations are not valid UTF-8.
    NonUtf8Abbr,
    /// The source bytes is too short to parse the content.
    DataTooShort,
    /// Invalid time zone file name.
    InvalidTimeZoneFileName,
    /// The time zone transition type is invalid.
    InvalidType,
    /// Name offset is out of bounds.
    NameOffsetOutOfBounds,
}

impl Header {
    /// Parses the header from the prefix of the `source`.
    fn parse(source: &[u8]) -> Result<Self, Error> {
        if source.len() < Self::HEADER_LEN {
            return Err(Error::HeaderTooShort);
        }
        if source[..4] != MAGIC {
            return Err(Error::InvalidMagic);
        }
        match source[4] {
            b'2' | b'3' => {}
            _ => return Err(Error::UnsupportedVersion),
        }
        let tzh_ttisgmtcnt = BE::read_u32(&source[20..24]) as usize;
        let tzh_ttisstdcnt = BE::read_u32(&source[24..28]) as usize;
        let tzh_leapcnt = BE::read_u32(&source[28..32]) as usize;
        let tzh_timecnt = BE::read_u32(&source[32..36]) as usize;
        let tzh_typecnt = BE::read_u32(&source[36..40]) as usize;
        let tzh_charcnt = BE::read_u32(&source[40..44]) as usize;

        if tzh_typecnt == 0 {
            return Err(Error::NoTypes);
        }

        Ok(Header {
            tzh_leapcnt,
            tzh_timecnt,
            tzh_typecnt,
            tzh_charcnt,
        })
    }

    // The length of the header.
    const HEADER_LEN: usize = 44;

    /// The length of the content, when `time_t` is represented by type `L`.
    fn data_len<L>(&self) -> usize {
        self.tzh_timecnt * (size_of::<L>() + 1)
            + self.tzh_typecnt * 8
            + self.tzh_charcnt
            + self.tzh_leapcnt * (size_of::<L>() + 4)
    }

    /// Parses the time zone information from the prefix of `content`.
    fn parse_content(&self, content: &[u8]) -> Result<String, Error> {
        // Obtain the byte indices where each array ends.
        let trans_encoded_end = self.tzh_timecnt * 8;
        let local_time_types_end = trans_encoded_end + self.tzh_timecnt;
        let infos_end = local_time_types_end + self.tzh_typecnt * 6;
        let abbr_end = infos_end + self.tzh_charcnt;

        // Collect the timezone abbreviations.
        let names = from_utf8(&content[infos_end..abbr_end]).map_err(|_| Error::NonUtf8Abbr)?;
        Ok(names.to_string())
    }
}