extern crate rstzfile;
use std::{env, convert::TryInto};
use rstzfile::*;
use chrono::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let requested_timezone = &args[1];
    let year = &args[2];
    parse(&requested_timezone, year.parse().unwrap());
}

fn parse(requested_timezone: &str, year: i32) {
    // Opens TZfile
    let buffer = match Tzfile::read(&requested_timezone) {
        Ok(b) => b,
        Err(e) => { println!("{}",e) ; return }
    };

    // Parses TZfile header
    let header = Tzfile::parse_header(&buffer);

    // Parses file content
    let timezone = match header {
        Ok(h) => h.parse(&buffer),
        Err(e) => { println!("{}",e) ; return }
    };

    println!("{:?}", timezone);

    let mut timechanges = Vec::new();
    let mut nearest_timechange: i32 = 0;
    let currentyearbeg = Utc.ymd(year, 1, 1).and_hms(0, 0, 0);
    let currentyearend = Utc.ymd(year, 12, 31).and_hms(0, 0, 0);
    
    for t in 0..timezone.tzh_timecnt_data.len() {
        // Get timechanges for selected year
        if timezone.tzh_timecnt_data[t] > currentyearbeg && timezone.tzh_timecnt_data[t] < currentyearend { timechanges.push(t); }
        if timezone.tzh_timecnt_data[t] < currentyearbeg { nearest_timechange = t.try_into().unwrap(); };
    }
    
    if timechanges.len() !=0 { println!("Time changes for specified year at index : {:?}", timechanges) }
    else { println!("Latest time change for specified year at index : {:?}", nearest_timechange) };
}
