extern crate rstzfile;
use chrono::prelude::*;
use rstzfile::*;
use std::convert::TryInto;

#[derive(Debug, PartialEq, Eq, Clone)]
struct Tzdata {
    dst_from: Option<DateTime<Utc>>,
    dst_until: Option<DateTime<Utc>>,
    raw_offset: FixedOffset,
    dst_offset: FixedOffset,
    abbreviation: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Timechange {
    time: DateTime<Utc>,
    gmtoff: FixedOffset,
    isdst: bool,
    abbreviation: String
}

pub fn export(requested_timezone: &str, year: i32) {
    // Opens TZfile
    let buffer = match Tzfile::read(&requested_timezone) {
        Ok(b) => b,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    // Parses TZfile header
    let header = Tzfile::parse_header(&buffer);

    // Parses file content
    let timezone = match header {
        Ok(h) => h.parse(&buffer),
        Err(e) => {
            println!("{}", e);
            return;
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
            let 
            println!(
                "{:?} {:?}",
                timezone.tzh_timecnt_data[timechanges[t]],
                timezone.tzh_typecnt[timezone.tzh_timecnt_indices[timechanges[t]] as usize]
            );
        }
    } else {
        //println!("Latest time change for specified year at index : {:?}", nearest_timechange);
        println!(
            "{:?} {:?}",
            timezone.tzh_timecnt_data[nearest_timechange],
            timezone.tzh_typecnt[timezone.tzh_timecnt_indices[nearest_timechange] as usize]
        );
    };
}

