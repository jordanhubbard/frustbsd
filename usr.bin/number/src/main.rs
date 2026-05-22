// SPDX-License-Identifier: BSD-2-Clause
// Translation of FreeBSD usr.bin/number/number.c to Rust

use std::io::{self, Read};
use std::process;

const MAXNUM: usize = 65;

static NAME1: &[&str] = &[
    "", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen",
    "seventeen", "eighteen", "nineteen",
];

static NAME2: &[&str] = &[
    "", "ten", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "nineteen",
];

static NAME3: &[&str] = &[
    "hundred", "thousand", "million", "billion", "trillion", "quadrillion",
    "quintillion", "sextillion", "septillion", "octillion", "nonillion", "decillion",
    "undecillion", "duodecillion", "tredecillion", "quattuordecillion",
    "quindecillion", "sexdecillion", "septendecillion", "octodecillion",
    "novemdecillion", "vigintillion",
];

static PREF: &[&str] = &["", "ten-", "hundred-"];

fn errx(code: i32, msg: &str) -> ! {
    eprintln!("number: {}", msg);
    process::exit(code);
}

fn usage() -> ! {
    eprintln!("usage: number [-l] [# ...]");
    process::exit(1);
}

fn number_fn(p: &[u8], len: usize) -> bool {
    let mut rval = false;
    match len {
        3 => {
            if p[0] != b'0' {
                rval = true;
                print!("{} hundred", NAME1[(p[0] - b'0') as usize]);
            }
            // Fall through to 2-digit handling
            let p2 = &p[1..];
            let val = (p2[1] - b'0') as usize + (p2[0] - b'0') as usize * 10;
            if val != 0 {
                if rval {
                    print!(" ");
                }
                if val < 20 {
                    print!("{}", NAME1[val]);
                } else {
                    print!("{}", NAME2[val / 10]);
                    if val % 10 != 0 {
                        print!("-{}", NAME1[val % 10]);
                    }
                }
                rval = true;
            }
        }
        2 => {
            let val = (p[1] - b'0') as usize + (p[0] - b'0') as usize * 10;
            if val != 0 {
                if rval {
                    print!(" ");
                }
                if val < 20 {
                    print!("{}", NAME1[val]);
                } else {
                    print!("{}", NAME2[val / 10]);
                    if val % 10 != 0 {
                        print!("-{}", NAME1[val % 10]);
                    }
                }
                rval = true;
            }
        }
        1 => {
            if p[0] != b'0' {
                rval = true;
                print!("{}", NAME1[(p[0] - b'0') as usize]);
            }
        }
        _ => {}
    }
    rval
}

fn unit(len: usize, p: &[u8], lflag: bool) -> bool {
    let mut rval = false;
    let mut len = len;
    let mut p = p;

    if len > 3 {
        if len % 3 != 0 {
            let off = len % 3;
            len -= off;
            if number_fn(p, off) {
                rval = true;
                print!(" {}{}", NAME3[len / 3], if lflag { " " } else { ".\n" });
            }
            p = &p[off..];
        }
        while len > 3 {
            len -= 3;
            if number_fn(p, 3) {
                rval = true;
                print!(" {}{}", NAME3[len / 3], if lflag { " " } else { ".\n" });
            }
            p = &p[3..];
        }
    }
    if number_fn(p, len) {
        if !lflag {
            print!(".\n");
        }
        rval = true;
    }
    rval
}

fn pfract(len: usize) {
    match len {
        1 => print!("tenths.\n"),
        2 => print!("hundredths.\n"),
        _ => print!("{}{}ths.\n", PREF[len % 3], NAME3[len / 3]),
    }
}

fn convert(line: &str, lflag: bool) {
    let line_bytes = line.as_bytes();
    let mut fraction: Option<&[u8]> = None;
    let mut flen = 0usize;

    // Find the end of the number (newline or null)
    let mut end = line_bytes.len();
    for (i, &b) in line_bytes.iter().enumerate() {
        if b == b'\n' {
            end = i;
            break;
        }
    }

    // Check for leading blanks
    let mut start = 0;
    while start < end && (line_bytes[start] == b' ' || line_bytes[start] == b'\t') {
        start += 1;
    }

    // Validate characters
    for &b in &line_bytes[start..end] {
        if b.is_ascii_digit() {
            continue;
        }
        match b {
            b'.' => {
                if fraction.is_some() {
                    errx(1, &format!("illegal number: {}", line));
                }
                let dot_pos = start + (&line_bytes[start..]).iter().position(|&x| x == b'.').unwrap();
                fraction = Some(&line_bytes[dot_pos + 1..end]);
            }
            b'-' => {
                // Only allowed at the very start
                let dash_pos = start + (&line_bytes[start..]).iter().position(|&x| x == b'-').unwrap();
                if dash_pos != start {
                    errx(1, &format!("illegal number: {}", line));
                }
            }
            _ => {
                errx(1, &format!("illegal number: {}", line));
            }
        }
    }

    let int_part = &line_bytes[start..end];
    let int_str = std::str::from_utf8(int_part).unwrap_or("");

    // Split into integer and fraction parts
    let (int_digits, frac_digits) = if let Some(frac) = fraction {
        let dot_idx = int_str.find('.').unwrap();
        let int_s = &int_str[..dot_idx];
        let frac_s = std::str::from_utf8(frac).unwrap_or("");
        (int_s, frac_s)
    } else {
        (int_str, "")
    };

    let len = int_digits.len();
    flen = frac_digits.len();

    if len > MAXNUM || (frac_digits.len() > 0 && flen > MAXNUM) {
        errx(1, &format!("number too large, max {} digits.", MAXNUM));
    }

    let mut line_ptr = int_digits.as_bytes();
    let mut rval = false;

    if line_ptr.first() == Some(&b'-') {
        print!("minus{}", if lflag { " " } else { ".\n" });
        line_ptr = &line_ptr[1..];
    }

    let len = line_ptr.len();
    if len > 0 {
        rval = unit(len, line_ptr, lflag);
    }

    if !frac_digits.is_empty() {
        // Check if fraction has any non-zero digits
        let has_nonzero = frac_digits.bytes().any(|b| b != b'0');
        if has_nonzero {
            if rval {
                print!("and{}", if lflag { " " } else { ".\n" });
            }
            if unit(flen, frac_digits.as_bytes(), lflag) {
                if lflag {
                    print!(" ");
                }
                pfract(flen);
                rval = true;
            }
        }
    }

    if !rval {
        print!("zero{}", if lflag { "" } else { ".\n" });
    }
    if lflag {
        print!("\n");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut lflag = false;
    let mut i = 1;

    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with('-') && arg.len() > 1 {
            match arg.as_str() {
                "-l" => {
                    lflag = true;
                    i += 1;
                }
                _ => {
                    usage();
                }
            }
        } else {
            break;
        }
    }

    let remaining: Vec<&String> = args[i..].iter().collect();

    if remaining.is_empty() {
        // Read from stdin
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).unwrap_or(0);
        let mut first = true;
        for line in input.lines() {
            let line_with_nl = format!("{}\n", line);
            if line_with_nl.len() > 256 {
                errx(1, "line too long.");
            }
            if !first {
                print!("...\n");
            }
            convert(&line_with_nl, lflag);
            first = false;
        }
    } else {
        let mut first = true;
        for arg in &remaining {
            if !first {
                print!("...\n");
            }
            convert(arg, lflag);
            first = false;
        }
    }
}