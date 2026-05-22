/*-
 * SPDX-License-Identifier: BSD-3-Clause
 *
 * Copyright (c) 2000 Dan Papasian.  All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * 3. The name of the author may not be used to endorse or promote products
 *    derived from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY EXPRESS OR
 * IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
 * OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
 * IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY DIRECT, INDIRECT,
 * INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
 * NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
 * THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

use std::env;
use std::ffi::CString;
use std::os::unix::fs::MetadataExt;
use std::process;

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

static mut SILENT: bool = false;
static mut ALLPATHS: bool = false;

extern "C" {
    fn access(path: *const i8, amode: i32) -> i32;
    fn getuid() -> u32;
}

const X_OK: i32 = 1;

fn usage() {
    eprintln!("usage: which [-as] program ...");
    process::exit(EXIT_FAILURE);
}

fn is_there(candidate: &str) -> bool {
    let c_candidate = match CString::new(candidate) {
        Ok(c) => c,
        Err(_) => return false,
    };

    // XXX work around access(2) false positives for superuser
    unsafe {
        if access(c_candidate.as_ptr(), X_OK) == 0 {
            if let Ok(meta) = std::fs::metadata(candidate) {
                if meta.is_file() {
                    let uid = getuid();
                    let mode = meta.mode();
                    let s_ixusr: u32 = 0o100;
                    let s_ixgrp: u32 = 0o010;
                    let s_ixoth: u32 = 0o001;
                    if uid != 0 || (mode & (s_ixusr | s_ixgrp | s_ixoth)) != 0 {
                        if !SILENT {
                            println!("{}", candidate);
                        }
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn print_matches(path_env: &str, filename: &str) -> bool {
    // If filename contains '/', check it directly
    if filename.contains('/') {
        return is_there(filename);
    }

    let mut found = false;
    for d in path_env.split(':') {
        let dir = if d.is_empty() { "." } else { d };
        let candidate = format!("{}/{}", dir, filename);
        if is_there(&candidate) {
            found = true;
            if !unsafe { ALLPATHS } {
                break;
            }
        }
    }
    found
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut silent = false;
    let mut allpaths = false;

    let mut i = 1;
    let mut file_args: Vec<String> = Vec::new();

    while i < args.len() {
        let arg = &args[i];
        if arg == "-a" {
            allpaths = true;
            i += 1;
        } else if arg == "-s" {
            silent = true;
            i += 1;
        } else if arg.starts_with('-') {
            // Handle combined short options
            let mut j = 1;
            while j < arg.len() {
                let ch = arg.as_bytes()[j];
                match ch {
                    b'a' => allpaths = true,
                    b's' => silent = true,
                    _ => usage(),
                }
                j += 1;
            }
            i += 1;
        } else {
            file_args.push(arg.clone());
            i += 1;
        }
    }

    if file_args.is_empty() {
        usage();
    }

    unsafe {
        SILENT = silent;
        ALLPATHS = allpaths;
    }

    let path_env = match env::var("PATH") {
        Ok(p) => p,
        Err(_) => process::exit(EXIT_FAILURE),
    };

    let mut status = EXIT_SUCCESS;

    for filename in &file_args {
        if filename.len() >= 256 {
            // FILENAME_MAX approximation
            status = EXIT_FAILURE;
            continue;
        }
        if !print_matches(&path_env, filename) {
            status = EXIT_FAILURE;
        }
    }

    process::exit(status);
}