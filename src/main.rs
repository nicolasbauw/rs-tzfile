mod tz;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let requested_timezone = &args[1];
    let year = &args[2];
    tz::export(&requested_timezone, year.parse().unwrap());
}
