# libtzfile

[![Current Crates.io Version](https://img.shields.io/crates/v/libtzfile.svg)](https://crates.io/crates/libtzfile)
[![Downloads badge](https://img.shields.io/crates/d/libtzfile.svg)](https://crates.io/crates/libtzfile)


This low-level library reads the system timezone information files and returns a Tz struct representing the TZfile
fields as described in the man page (<http://man7.org/linux/man-pages/man5/tzfile.5.html>).
Parses V2 (64 bits) format version since 1.0.0, so be aware that the tzh_timecnt_data field is now a `Vec<i64>`.

For higher level parsing, see [my high-level parsing library](https://crates.io/crates/tzparse).

To keep the low-level aspect of the library, since 0.5.0 chrono is an optional feature which is not enabled by default, so tzh_timecnt_data is now the raw `i64` timestamp.
For libtzfile to return tzh_timecnt_data as `DateTime<Utc>`, you can add this in Cargo.toml:
```
[dependencies.libtzfile]
version = "1.0.0"
features = ["with-chrono"]
```
Here is an example:
```rust
extern crate libtzfile;

fn main() {
    println!("{:?}", libtzfile::parse("America/Phoenix").unwrap());
}
```

which outputs (with chrono enabled):
```
Tz { tzh_timecnt_data: [1883-11-18T19:00:00Z, 1918-03-31T09:00:00Z, 1918-10-27T08:00:00Z,
1919-03-30T09:00:00Z, 1919-10-26T08:00:00Z, 1942-02-09T09:00:00Z, 1944-01-01T06:01:00Z,
1944-04-01T07:01:00Z, 1944-10-01T06:01:00Z, 1967-04-30T09:00:00Z, 1967-10-29T08:00:00Z],
tzh_timecnt_indices: [2, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2],
tzh_typecnt: [Ttinfo { tt_gmtoff: -26898, tt_isdst: 0, tt_abbrind: 0 },
Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 1 }, Ttinfo { tt_gmtoff: -25200, tt_isdst: 0, tt_abbrind: 2 },
Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 3 }], tz_abbr: ["LMT", "MDT", "MST", "MWT"] }
```

By default, with chrono disabled, tzh_timecnt_data will be like:
```
tzh_timecnt_data: [FFFFFFFF5E040CB0, FFFFFFFF9EA63A90, FFFFFFFF9FBB0780,
FFFFFFFFA0861C90, FFFFFFFFA19AE980, FFFFFFFFCB890C90, FFFFFFFFCF17DF1C,
FFFFFFFFCF8FE5AC, FFFFFFFFD0811A1C, FFFFFFFFFAF87510, FFFFFFFFFBE85800]
```
It uses system TZfiles (default location on Linux and Macos /usr/share/zoneinfo). On Windows, default expected location is HOME/.zoneinfo. You can override the TZfiles default location with the TZFILES_DIR environment variable. Example for Windows:

$env:TZFILES_DIR="C:\Users\nbauw\Dev\rs-tzfile\zoneinfo\"; cargo run

The tests (cargo test) are written to match [2019c version of timezone database](https://www.iana.org/time-zones).

License: GPL-3.0
