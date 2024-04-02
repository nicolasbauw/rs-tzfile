use crate::*;
extern crate std;
#[cfg(target_family = "unix")]
static TIMEZONE: &str = "/usr/share/zoneinfo/America/Phoenix";

#[test]
fn parse_hdr() {
    let buf = std::fs::read(TIMEZONE).unwrap();
    let hdr = Header {
        tzh_ttisutcnt: 5,
        tzh_ttisstdcnt: 5,
        tzh_leapcnt: 0,
        tzh_timecnt: 11,
        tzh_typecnt: 5,
        tzh_charcnt: 16,
        v2_header_start: 155,
    };
    assert_eq!(Tz::parse_header(&buf).unwrap(), hdr);
}

#[test]
fn parse_indices() {
    let buf = std::fs::read(TIMEZONE).unwrap();
    let indices: [u8; 11] = [4, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2];
    assert_eq!(Tz::new(buf).unwrap().tzh_timecnt_indices, indices);
}

#[test]
fn parse_timedata() {
    let buf = std::fs::read(TIMEZONE).unwrap();
    let data: Vec<i64> = std::vec![
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
    assert_eq!(Tz::new(buf).unwrap().tzh_timecnt_data, data);
}

#[test]
fn parse_ttutoff() {
    let buf = std::fs::read(TIMEZONE).unwrap();
    let data: [isize; 5] = [-26898, -21600, -25200, -21600, -25200];
    let c: Vec<isize> = Tz::new(buf)
        .unwrap()
        .tzh_typecnt
        .iter()
        .map(|ttinfo| ttinfo.tt_utoff)
        .collect();
    assert_eq!(c, data);
}

#[test]
fn parse_abbr() {
    let buf = std::fs::read(TIMEZONE).unwrap();
    let abbr: Vec<String> = std::vec!["LMT", "MDT", "MST", "MWT"]
        .iter()
        .map(|x| x.to_string())
        .collect();
    assert_eq!(Tz::new(buf).unwrap().tz_abbr, abbr);
}
