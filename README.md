# libtzfile

[![Current Crates.io Version](https://img.shields.io/crates/v/libtzfile.svg)](https://crates.io/crates/libtzfile)
[![Downloads badge](https://img.shields.io/crates/d/libtzfile.svg)](https://crates.io/crates/libtzfile)

This library reads the system timezone information files provided by IANA and returns a Tz struct representing the TZfile
fields as described in the man page (<http://man7.org/linux/man-pages/man5/tzfile.5.html>).

For higher level parsing, see [my high-level parsing library](https://crates.io/crates/tzparse).

```rust
fn main() {
    println!("{:?}", libtzfile::parse("/usr/share/zoneinfo/America/Phoenix").unwrap());
}
```

```text
Tz { tzh_timecnt_data: [FFFFFFFF5E040CB0, FFFFFFFF9EA63A90, FFFFFFFF9FBB0780,
FFFFFFFFA0861C90, FFFFFFFFA19AE980, FFFFFFFFCB890C90, FFFFFFFFCF17DF1C,
FFFFFFFFCF8FE5AC, FFFFFFFFD0811A1C, FFFFFFFFFAF87510, FFFFFFFFFBE85800],
tzh_timecnt_indices: [2, 1, 2, 1, 2, 3, 2, 3, 2, 1, 2],
tzh_typecnt: [Ttinfo { tt_gmtoff: -26898, tt_isdst: 0, tt_abbrind: 0 },
Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 1 }, Ttinfo { tt_gmtoff: -25200, tt_isdst: 0, tt_abbrind: 2 },
Ttinfo { tt_gmtoff: -21600, tt_isdst: 1, tt_abbrind: 3 }], tz_abbr: ["LMT", "MDT", "MST", "MWT"] }
```

The tests (cargo test) are written to match [2019c version of timezone database](https://data.iana.org/time-zones/tz-link.html).

License: GPL-3.0
