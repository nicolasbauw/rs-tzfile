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
        tzh_ttisgmtcnt: 5,
        tzh_ttisstdcnt: 5,
        tzh_leapcnt: 0,
        tzh_timecnt: 11,
        tzh_typecnt: 5,
        tzh_charcnt: 16,
        v2_header_start: 155,
    };
    assert_eq!(Tz::parse_header(&buf).unwrap(), amph);
}

#[test]
fn parse_indices() {
    let amph: [u8; 11] = [4, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2];
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
    let amph: [isize; 5] = [-26898, -21600, -25200, -21600, -25200];
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
    let tztest = Tz::new("/usr/share/zoneinfo/Europe/Paris")
        .unwrap()
        .zoneinfo()
        .unwrap();
    #[cfg(target_os = "windows")]
    let tztest = (Tz::new("c:\\Users\\nbauw\\Dev\\zoneinfo\\Europe\\Paris").unwrap())
        .zoneinfo()
        .unwrap();
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
    assert_eq!(
        Err(crate::TzError::NoData),
        Tz::new(timezone).unwrap().transition_times(None)
    );
}
