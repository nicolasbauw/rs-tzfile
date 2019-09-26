/* This module's get function is used to retrieve
time changes and characteristics for a given TZ.
See function comments for output sample. */

extern crate tzfile;
use chrono::prelude::*;
use tzfile::*;
use std::convert::TryInto;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tzdata {
    dst_from: Option<DateTime<Utc>>,
    dst_until: Option<DateTime<Utc>>,
    raw_offset: isize,
    dst_offset: isize,
    utc_offset: FixedOffset,
    abbreviation: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Timechange {
    time: DateTime<Utc>,
    gmtoff: isize,
    isdst: bool,
    abbreviation: String
}

/* Returns Option enum of Tzdata struct, output sample:
dst_from: Some(2019-03-31T01:00:00Z), dst_until: Some(2019-10-27T01:00:00Z), raw_offset: +01:00, dst_offset: +02:00, abbreviation: "CEST"*/

pub fn get(requested_timezone: &str, year: i32) -> Option<Tzdata> {
    // Opens TZfile
    let buffer = match Tzfile::read(&requested_timezone) {
        Ok(b) => b,
        Err(_e) => {
            //println!("{}", e);
            return None;
        }
    };

    // Parses TZfile header
    let header = Tzfile::parse_header(&buffer);

    // Parses file content
    let timezone = match header {
        Ok(h) => h.parse(&buffer),
        Err(_e) => {
            //println!("{}", e);
            return None;
        }
    };

    //println!("{:?}", timezone);

    // used to store timechange indices
    let mut timechanges = Vec::new();
    let mut nearest_timechange: usize = 0;

    // Used to store parsed useful data
    let mut parsedtimechanges = Vec::new();

    // for year comparison
    let currentyearbeg = Utc.ymd(year, 1, 1).and_hms(0, 0, 0);
    let currentyearend = Utc.ymd(year, 12, 31).and_hms(0, 0, 0);

    // Get and store the timechange indices
    for t in 0..timezone.tzh_timecnt_data.len() {
        if timezone.tzh_timecnt_data[t] > currentyearbeg
            && timezone.tzh_timecnt_data[t] < currentyearend
        {
            timechanges.push(t);
        }
        if timezone.tzh_timecnt_data[t] < currentyearbeg {
            nearest_timechange = t.try_into().unwrap();
        };
    }

    if timechanges.len() != 0 {
        //println!("Time changes for specified year at index : {:?}", timechanges);
        for t in 0..timechanges.len() {
            let tc = Timechange {
                time: timezone.tzh_timecnt_data[timechanges[t]],
                gmtoff: timezone.tzh_typecnt[timezone.tzh_timecnt_indices[timechanges[t]] as usize].tt_gmtoff,
                isdst: timezone.tzh_typecnt[timezone.tzh_timecnt_indices[timechanges[t]] as usize].tt_isdst == 1,
                abbreviation: timezone.tz_abbr[timezone.tzh_typecnt[timezone.tzh_timecnt_indices[timechanges[t]] as usize].tt_abbrind as usize].to_string(),
            };
            parsedtimechanges.push(tc);
        }
    } else {
        let tc = Timechange {
                time: timezone.tzh_timecnt_data[nearest_timechange],
                gmtoff: timezone.tzh_typecnt[timezone.tzh_timecnt_indices[nearest_timechange] as usize].tt_gmtoff,
                isdst: timezone.tzh_typecnt[timezone.tzh_timecnt_indices[nearest_timechange] as usize].tt_isdst == 1,
                abbreviation: timezone.tz_abbr[timezone.tzh_typecnt[timezone.tzh_timecnt_indices[nearest_timechange] as usize].tt_abbrind as usize].to_string(),
        };
        parsedtimechanges.push(tc);
    }
    //Some(parsedtimechanges)

    let d = Utc::now();
    if parsedtimechanges.len() == 2 {
        // 2 times changes the same year ? DST observed
        // Are we in a dst period ? true / false
        let dst = d > parsedtimechanges[0].time
            && d < parsedtimechanges[1].time;
        //println!("{}", dst);
        Some(Tzdata {
        dst_from: Some(parsedtimechanges[0].time),
        dst_until: Some(parsedtimechanges[1].time),
        raw_offset: parsedtimechanges[1].gmtoff,
        dst_offset: parsedtimechanges[0].gmtoff,
        utc_offset: if dst == true { FixedOffset::east(parsedtimechanges[0].gmtoff as i32) } else { FixedOffset::east(parsedtimechanges[1].gmtoff as i32) },
        abbreviation: if dst == true { parsedtimechanges[0].abbreviation.clone() } else { parsedtimechanges[1].abbreviation.clone() },
        })
    } else if parsedtimechanges.len()==1 {
        Some(Tzdata {
        dst_from: None,
        dst_until: None,
        raw_offset: parsedtimechanges[0].gmtoff,
        dst_offset: 0,
        utc_offset: FixedOffset::east(parsedtimechanges[0].gmtoff as i32),
        abbreviation: parsedtimechanges[0].abbreviation.clone(),
        })
    } else { None }
}

