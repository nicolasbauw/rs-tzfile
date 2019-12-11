# TZfile reading library

[![Current Crates.io Version](https://img.shields.io/crates/v/libtzfile.svg)](https://crates.io/crates/libtzfile)

This low-level library parses the system timezone information files and returns a Tz struct representing the TZfile fields as described in the man page (http://man7.org/linux/man-pages/man5/tzfile.5.html)

To keep the low-level aspect, since 0.5.0 chrono is an optional feature which is not enabled by default, so tzh_timecnt_data is now the raw `i32` timestamp.
For libtzfile to return tzh_timecnt_data as `DateTime<Utc>`, you can either use the version 0.4.0 of libtzfile, or add this in Cargo.toml:
```
[dependencies.libtzfile]
version = "0.5.1"
features = ["with-chrono"]
```

Only compatible with V1 (32 bits) format version for the moment.

Here is an output sample with chrono enabled:

```
{ tzh_timecnt_data: [1901-12-13T20:45:52Z, 1918-03-31T09:00:00Z, 1918-10-27T08:00:00Z,
 1919-03-30T09:00:00Z, 1919-10-26T08:00:00Z, 1942-02-09T09:00:00Z, 1944-01-01T06:01:00Z,
  1944-04-01T07:01:00Z, 1944-10-01T06:01:00Z, 1967-04-30T09:00:00Z, 1967-10-29T08:00:00Z],
   tzh_timecnt_indices: [2, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2], tzh_typecnt: [Ttinfo { tt_gmtoff: -26898, tt_isdst: 0, tt_abbrind: 0 },
Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 1 }, Ttinfo { tt_gmtoff: -25200, tt_isdst: 0, tt_abbrind: 2 },
Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 3 }], tz_abbr: ["LMT", "MDT", "MST", "MWT"] }
```

## How to use:

Insert this in cargo.toml:
````
[dependencies]
libtzfile = "0.5.1"
````

A basic example:

```
extern crate libtzfile;

fn main() {
    println!("{:?}", libtzfile::parse("America/Phoenix").expect("Timezone not found"));
}
```

It uses system TZfiles (default location on Linux and Macos /usr/share/zoneinfo). On Windows, default expected location is HOME/.zoneinfo. You can override the TZfiles default location with the TZFILES_DIR environment variable. Example for Windows:

```
$env:TZFILES_DIR="C:\Users\nbauw\Dev\rs-tzfile\zoneinfo\"; cargo run
```