// SPDX-License-Identifier: BSD-2-Clause
// Translation of FreeBSD usr.bin/pom/pom.c to Rust

use std::env;
use std::process;

const PI: f64 = 3.14159265358979323846;
const EPOCH: i32 = 85;
const EPSILON_G: f64 = 279.611371;
const RHO_G: f64 = 282.680403;
const ECCEN: f64 = 0.01671542;
const LZERO: f64 = 18.251907;
const PZERO: f64 = 192.917585;
const NZERO: f64 = 55.204723;

fn isleap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn adj360(deg: &mut f64) {
    loop {
        if *deg < 0.0 {
            *deg += 360.0;
        } else if *deg > 360.0 {
            *deg -= 360.0;
        } else {
            break;
        }
    }
}

fn dtor(deg: f64) -> f64 {
    deg * PI / 180.0
}

fn potm(days: f64) -> f64 {
    let mut n = 360.0 * days / 365.2422;
    adj360(&mut n);
    let mut msol = n + EPSILON_G - RHO_G;
    adj360(&mut msol);
    let ec = 360.0 / PI * ECCEN * (dtor(msol)).sin();
    let mut lambda_sol = n + ec + EPSILON_G;
    adj360(&mut lambda_sol);
    let mut l = 13.1763966 * days + LZERO;
    adj360(&mut l);
    let mut mm = l - (0.1114041 * days) - PZERO;
    adj360(&mut mm);
    let mut _nm = NZERO - (0.0529539 * days);
    adj360(&mut _nm);
    let ev = 1.2739 * (dtor(2.0 * (l - lambda_sol) - mm)).sin();
    let ac = 0.1858 * (dtor(msol)).sin();
    let a3 = 0.37 * (dtor(msol)).sin();
    let mm_prime = mm + ev - ac - a3;
    let ec = 6.2886 * (dtor(mm_prime)).sin();
    let a4 = 0.214 * (dtor(2.0 * mm_prime)).sin();
    let l_prime = l + ev + ec - ac + a4;
    let v = 0.6583 * (dtor(2.0 * (l_prime - lambda_sol))).sin();
    let ld_prime = l_prime + v;
    let d = ld_prime - lambda_sol;
    50.0 * (1.0 - (dtor(d)).cos())
}

fn usage(progname: &str) {
    eprintln!("Usage: {} [-p] [-d yyyy.mm.dd] [-t hh:mm:ss]", progname);
    process::exit(64);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let progname = &args[0];

    let mut pflag = false;
    let mut odate: Option<String> = None;
    let mut otime: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-d" => {
                i += 1;
                if i < args.len() {
                    odate = Some(args[i].clone());
                } else {
                    usage(progname);
                }
            }
            "-p" => {
                pflag = true;
            }
            "-t" => {
                i += 1;
                if i < args.len() {
                    otime = Some(args[i].clone());
                } else {
                    usage(progname);
                }
            }
            _ => {
                usage(progname);
            }
        }
        i += 1;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let mut tt = now.as_secs() as i64;

    if odate.is_some() || otime.is_some() {
        let mut sec = tt % 60;
        let mut min = (tt / 60) % 60;
        let mut hour = (tt / 3600) % 24;
        let mut total_days = tt / 86400;

        let mut year = 1970i64;
        loop {
            let days_in_year = if isleap(year as i32) { 366 } else { 365 };
            if total_days < days_in_year {
                break;
            }
            total_days -= days_in_year;
            year += 1;
        }

        let month_days: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let leap = isleap(year as i32);
        let mut md = month_days;
        if leap {
            md[1] = 29;
        }

        let mut doy = 0i64;
        let mut m = 0;
        while m < 12 {
            if total_days < md[m] as i64 {
                break;
            }
            doy += md[m] as i64;
            m += 1;
        }

        let mut tm_year = (year - 1900) as i32;
        let mut tm_mon = m as i32;
        let mut tm_mday = total_days as i32 + 1;

        if let Some(ref d) = odate {
            let parts: Vec<&str> = d.split('.').collect();
            if parts.len() == 3 {
                tm_year = parts[0].parse::<i32>().unwrap_or(tm_year) - 1900;
                tm_mon = parts[1].parse::<i32>().unwrap_or(tm_mon) - 1;
                tm_mday = parts[2].parse::<i32>().unwrap_or(tm_mday);
            }
        }

        if let Some(ref t) = otime {
            let parts: Vec<&str> = t.split(':').collect();
            if parts.len() == 3 {
                hour = parts[0].parse::<i64>().unwrap_or(hour);
                min = parts[1].parse::<i64>().unwrap_or(min);
                sec = parts[2].parse::<i64>().unwrap_or(sec);
            }
        }

        let mut total_days2: i64 = 0;
        let target_year = tm_year + 1900;
        for y in 1970..target_year {
            total_days2 += if isleap(y) { 366 } else { 365 };
        }

        let leap_target = isleap(target_year);
        let mut md2 = month_days;
        if leap_target {
            md2[1] = 29;
        }
        for m in 0..(tm_mon as usize) {
            total_days2 += md2[m] as i64;
        }
        total_days2 += (tm_mday - 1) as i64;

        tt = total_days2 * 86400 + hour * 3600 + min * 60 + sec;
    }

    let mut sec = tt % 60;
    let mut min = (tt / 60) % 60;
    let mut hour = (tt / 3600) % 24;
    let mut total_days = tt / 86400;

    let mut year = 1970i64;
    loop {
        let days_in_year = if isleap(year as i32) { 366 } else { 365 };
        if total_days < days_in_year {
            break;
        }
        total_days -= days_in_year;
        year += 1;
    }

    let tm_year = year as i32 - 1900;
    let tm_yday = total_days as i32;

    let days = (tm_yday + 1) as f64
        + ((hour as f64 + min as f64 / 60.0 + sec as f64 / 3600.0) / 24.0);

    let mut days_acc = days;
    for cnt in EPOCH..tm_year {
        days_acc += if isleap(1900 + cnt) { 366.0 } else { 365.0 };
    }

    let today = potm(days_acc);

    if pflag {
        println!("{:.0}", today);
        process::exit(0);
    }

    print!("The Moon is ");
    if today >= 99.5 {
        println!("Full");
    } else if today < 0.5 {
        println!("New");
    } else {
        let tomorrow = potm(days_acc + 1.0);
        if today >= 49.5 && today < 50.5 {
            if tomorrow > today {
                println!("at the First Quarter");
            } else {
                println!("at the Last Quarter");
            }
        } else {
            let waxing = tomorrow > today;
            print!("{} ", if waxing { "Waxing" } else { "Waning" });
            if today > 50.0 {
                println!("Gibbous ({:.0}% of Full)", today);
            } else if today < 50.0 {
                println!("Crescent ({:.0}% of Full)", today);
            }
        }
    }

    process::exit(0);
}