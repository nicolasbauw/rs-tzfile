# libtzfile

[![Current Crates.io Version](https://img.shields.io/crates/v/libtzfile.svg)](https://crates.io/crates/libtzfile)
[![Downloads badge](https://img.shields.io/crates/d/libtzfile.svg)](https://crates.io/crates/libtzfile)

This library reads the system timezone information files provided by IANA and returns a Tz struct representing the TZfile
fields as described in the man page (<http://man7.org/linux/man-pages/man5/tzfile.5.html>).

For higher level parsing, see [my high-level parsing library](https://crates.io/crates/tzparse).

Here is an example:
```rust
fn main() {
    println!("{:?}", libtzfile::parse("/usr/share/zoneinfo/America/Phoenix").unwrap());
}
```

```
Tz { tzh_timecnt_data: [-2717643600, -1633273200, -1615132800, -1601823600, -1583683200, -880210800, -820519140, -812653140, -796845540, -84380400, -68659200],
tzh_timecnt_indices: [2, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2], tzh_typecnt: [Ttinfo { tt_gmtoff: -26898, tt_isdst: 0, tt_abbrind: 0 },
Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 1 }, Ttinfo { tt_gmtoff: -25200, tt_isdst: 0, tt_abbrind: 2 },
Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 3 }], tz_abbr: ["LMT", "MDT", "MST", "MWT"] }
```

The tests (cargo test) are written to match [2020a version of timezone database](https://data.iana.org/time-zones/tz-link.html).

License: GPL-3.0
