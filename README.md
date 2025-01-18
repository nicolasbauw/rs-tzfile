# libtzfile

[![Current Crates.io Version](https://img.shields.io/crates/v/libtzfile.svg)](https://crates.io/crates/libtzfile)
[![Current docs Version](https://docs.rs/libtzfile/badge.svg)](https://docs.rs/libtzfile)
[![Downloads badge](https://img.shields.io/crates/d/libtzfile.svg)](https://crates.io/crates/libtzfile)

This library reads and parses the system timezone information files (TZ Files) provided by IANA.

The default feature is `std`. With `default-features = false`, the crate is `no_std` and uses `alloc::vec`. In both cases the `new()` method returns a Tz struct containing the TZfile
fields as described in the man page (<http://man7.org/linux/man-pages/man5/tzfile.5.html>).

- with `no_std` the function signature is `new(buf: Vec<u8>)` where `buf` is the TZ File data

```
// no_std
[dependencies]
libtzfile = { version = "3.1.0", default-features = false }
```

```
let tzfile = include_bytes!("/usr/share/zoneinfo/America/Phoenix").to_vec();
let tz = Tz::new(tzfile).unwrap();
```

- with `std` which is the default feature the function signature is `new(tz: &str)` where `tz` is the TZ File name

```
// std is the default
[dependencies]
libtzfile = "3.1.0"
```

```
use libtzfile::Tz;
let tzfile: &str = "/usr/share/zoneinfo/America/Phoenix";
println!("{:?}", Tz::new(tzfile).unwrap());
```

```
Tz { tzh_timecnt_data: [-2717643600, -1633273200, -1615132800, -1601823600, -1583683200, -880210800, -820519140, -812653140, -796845540, -84380400, -68659200], tzh_timecnt_indices: [2, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2], tzh_typecnt: [Ttinfo { tt_utoff: -26898, tt_isdst: 0, tt_abbrind: 0 }, Ttinfo { tt_utoff: -21600, tt_isdst: 1, tt_abbrind: 1 }, Ttinfo { tt_utoff: -25200, tt_isdst: 0, tt_abbrind: 2 }, Ttinfo { tt_utoff: -21600, tt_isdst: 1, tt_abbrind: 3 }], tz_abbr: ["LMT", "MDT", "MST", "MWT"] }
```

For higher level parsing, you can enable the **parse** or **json** features.
For instance, to display 2020 DST transitions in France, you can use the transition_times method:

```
use libtzfile::Tz;
let tzfile: &str = "/usr/share/zoneinfo/Europe/Paris";
println!("{:?}", Tz::new(tzfile).unwrap().transition_times(Some(2020)).unwrap());
```

```
[TransitionTime { time: 2020-03-29T01:00:00Z, utc_offset: 7200, isdst: true, abbreviation: "CEST" }, TransitionTime { time: 2020-10-25T01:00:00Z, utc_offset: 3600, isdst: false, abbreviation: "CET" }]
```

If you want more complete information about the timezone, you can use the zoneinfo method, which returns a more complete structure:

```
use libtzfile::Tz;
let tzfile: &str = "/usr/share/zoneinfo/Europe/Paris";
println!("{:?}", Tz::new(tzfile).unwrap().zoneinfo().unwrap());
```

```
Tzinfo { timezone: "Europe/Paris", utc_datetime: 2020-09-05T16:41:44.279502100Z, datetime: 2020-09-05T18:41:44.279502100+02:00, dst_from: Some(2020-03-29T01:00:00Z), dst_until: Some(2020-10-25T01:00:00Z), dst_period: true, raw_offset: 3600, dst_offset: 7200, utc_offset: +02:00, abbreviation: "CEST", week_number: 36 }
```

This more complete structure implements the Serialize trait and can be transformed to a json string via a method of the json feature (which includes methods from the parse feature):

```
use libtzfile::{Tz, TzError};
let tzfile: &str = "/usr/share/zoneinfo/Europe/Paris";
let tz = Tz::new(tzfile)?
    .zoneinfo()?
    .to_json()?;
println!("{}", tz);
```

```
{"timezone":"Europe/Paris","utc_datetime":"2020-09-05T18:04:50.546668500Z","datetime":"2020-09-05T20:04:50.546668500+02:00","dst_from":"2020-03-29T01:00:00Z","dst_until":"2020-10-25T01:00:00Z","dst_period":true,"raw_offset":3600,"dst_offset":7200,"utc_offset":"+02:00","abbreviation":"CEST","week_number":36}
```

This feature is used in my [world time API](https://crates.io/crates/world-time-api).

The tests (`cargo test`, `cargo test --no-default-features` or `cargo test --features parse|json`) are working with the [2025a timezone database](https://data.iana.org/time-zones/tz-link.html).

License: MIT
