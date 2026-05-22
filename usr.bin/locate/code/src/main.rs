// SPDX-License-Identifier: BSD-3-Clause
//
// Copyright (c) 1995-2022 Wolfram Schneider <wosch@FreeBSD.org>
// Copyright (c) 1989, 1993
//	The Regents of the University of California.  All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
// 3. Neither the name of the University nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED.  IN NO EVENT SHALL THE REGENTS OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
// OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
// HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
// LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
// OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGE.

use std::io::{self, BufRead, Write};
use std::process;

const LOCATE_PATH_MAX: usize = 1024;
const NBG: usize = 128;
const OFFSET: i32 = 14;
const PARITY: u8 = 0o200; // 128
const SWITCH: u8 = 30;
const UMLAUT: u8 = 31;
const ASCII_MIN: u8 = 32;
const ASCII_MAX: u8 = 127;
const BGBUFSIZE: usize = NBG * 2;

fn usage() {
    eprintln!("usage: locate.code common_bigrams < list > squozen_list");
    process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        usage();
    }

    // Read bigram file
    let bigram_file = &args[1];
    let content = match std::fs::read_to_string(bigram_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("locate.code: {}: {}", bigram_file, e);
            process::exit(1);
        }
    };

    // First line is the bigram table
    let bigrams_line = match content.lines().next() {
        Some(l) => l,
        None => {
            eprintln!("locate.code: get bigram array");
            process::exit(1);
        }
    };

    // Write bigram table to stdout
    let stdout = io::stdout();
    let mut out = stdout.lock();

    // Write exactly BGBUFSIZE bytes for the bigram table
    let bigrams_bytes = bigrams_line.as_bytes();
    let write_len = bigrams_bytes.len().min(BGBUFSIZE);
    if out.write_all(&bigrams_bytes[..write_len]).is_err() {
        eprintln!("locate.code: stdout");
        process::exit(1);
    }
    // Pad with zeros if needed
    if write_len < BGBUFSIZE {
        let padding = vec![0u8; BGBUFSIZE - write_len];
        if out.write_all(&padding).is_err() {
            eprintln!("locate.code: stdout");
            process::exit(1);
        }
    }

    // Init lookup table
    let mut big: [[i32; 256]; 256] = [[-1; 256]; 256];
    let bigrams_bytes = bigrams_line.as_bytes();
    let mut i = 0u32;
    let mut cp = 0;
    while cp + 1 < bigrams_bytes.len() && bigrams_bytes[cp] != b'\0' {
        let c1 = bigrams_bytes[cp] as usize;
        let c2 = bigrams_bytes[cp + 1] as usize;
        big[c1][c2] = i as i32;
        i += 1;
        cp += 2;
    }

    let mut buf1 = vec![b' '; LOCATE_PATH_MAX];
    let mut buf2 = vec![0u8; LOCATE_PATH_MAX];
    let mut oldpath = &mut buf1;
    let mut path = &mut buf2;
    let mut oldcount: i32 = 0;

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => {
                eprintln!("locate.code: stdin");
                process::exit(1);
            }
        }

        // Copy line into path buffer
        let line_bytes = line.as_bytes();
        let len = line_bytes.len().min(LOCATE_PATH_MAX - 1);
        path[..len].copy_from_slice(&line_bytes[..len]);
        path[len] = b'\0';

        // Skip empty lines
        if path[0] == b'\n' {
            std::mem::swap(&mut path, &mut oldpath);
            continue;
        }

        // Remove newline
        for i in 0..len {
            if path[i] == b'\n' {
                path[i] = b'\0';
                break;
            }
        }

        // Skip longest common prefix
        let mut cp = 0;
        let mut old_cp = 0;
        while cp < len && old_cp < oldpath.len() && path[cp] == oldpath[old_cp] {
            if path[cp] == b'\0' {
                break;
            }
            cp += 1;
            old_cp += 1;
        }

        let count = (cp - 0) as i32;
        let diffcount = count - oldcount + OFFSET;
        oldcount = count;

        if diffcount < 0 || diffcount > 2 * OFFSET {
            // Switch code + out-of-range count as word (little-endian u16)
            if out.write_all(&[SWITCH]).is_err() {
                eprintln!("locate.code: stdout");
                process::exit(1);
            }
            let word = diffcount as u16;
            if out.write_all(&word.to_le_bytes()).is_err() {
                eprintln!("locate.code: stdout");
                process::exit(1);
            }
        } else {
            if out.write_all(&[diffcount as u8]).is_err() {
                eprintln!("locate.code: stdout");
                process::exit(1);
            }
        }

        // Encode the residue
        let mut cp = cp;
        while cp < len && path[cp] != b'\0' {
            let code = big[path[cp] as usize][if cp + 1 < len && path[cp + 1] != b'\0' { path[cp + 1] as usize } else { 0 }];

            if code != -1 {
                // Bigram found
                let encoded = ((code as u32) / 2) as u8 | PARITY;
                if out.write_all(&[encoded]).is_err() {
                    eprintln!("locate.code: stdout");
                    process::exit(1);
                }
                cp += 2;
            } else {
                // Print up to 2 characters individually
                for _ in 0..2 {
                    if cp >= len || path[cp] == b'\0' {
                        break;
                    }
                    if path[cp] < ASCII_MIN || path[cp] > ASCII_MAX {
                        // Umlaut / 8-bit character
                        if out.write_all(&[UMLAUT, path[cp]]).is_err() {
                            eprintln!("locate.code: stdout");
                            process::exit(1);
                        }
                    } else {
                        if out.write_all(&[path[cp]]).is_err() {
                            eprintln!("locate.code: stdout");
                            process::exit(1);
                        }
                    }
                    cp += 1;
                }
            }
        }

        // Swap pointers
        std::mem::swap(&mut path, &mut oldpath);
    }

    if out.flush().is_err() {
        eprintln!("locate.code: stdout");
        process::exit(1);
    }

    process::exit(0);
}