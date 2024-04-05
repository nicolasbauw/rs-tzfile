# Changelog

### 3.1.0 (2024-04-05)

- [Added] no_std support

### 3.0.0 (2024-03-27)

- [Changed] Updated some methods from chrono that became deprecated
- [Changed] Tests updated for 2024a system files
- [Changed] Updated fields tzh_ttisgmtcnt and tt_gmtoff to tzh_ttisutcnt and tt_utoff

### 2.0.2 (2021-07-08)

- [Fixed] Issue #3 (Calling zoneinfo on a file without transition times panics)
- [Added] Proper handling of TZFiles without transition times by the zoneinfo() function

### 2.0.0 (2020-09-07)

- [Added] Merged Functionalities from the (no longer updated) tzparse crate
- [Added] Error conversion for serde (json feature)
- [Changed] Renamed some fields, structs and functions, updated the doc accordingly

### 1.1.0 (2020-06-23)

- [Removed] Chrono feature
- [Removed] Tzfiles root dir override by ENV
- [Changed] Tzfiles are now accessed by absolute path, no longer by relative path
