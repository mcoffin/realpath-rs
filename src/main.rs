extern crate getopts;

use getopts::{optflag, getopts};
use std::iter;
use std::os;
use std::path::Path;

/// Contains version info
static VERSION: [uint, ..3] = [0, 0, 1];

/// Maximum number of symlinks to follow
static MAX_DEPTH: uint = 256;

mod realpath;

fn print_usage(invocation: &str, opts: &[getopts::OptGroup]) {
    let short = format!(
        "{} path...",
        getopts::short_usage(invocation.as_slice(), opts));
    println!(
        "{}",
        getopts::usage(short.as_slice(), opts));
}

fn print_results<T: iter::Iterator<Path>>(mut iter: T) {
    for path in iter {
        println!("{}", path.display());
    }
}

fn main() {
    let args: Vec<String> = os::args();

    let invocation = args[0].clone();

    let opts = [
        optflag("h", "help", "print usage info then exit"),
        optflag("v", "version", "print version info then exit"),
        optflag("s", "strip", "don't expand symlinks"),
    ];
    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    // If the help option is present, print usage info and exit
    if matches.opt_present("h") {
        print_usage(invocation.as_slice(), opts);
        return;
    }

    // If the version option is present, print version info and exit
    if matches.opt_present("v") {
        println!(
            "{name:s} v{major}.{minor}.{micro}",
            name=invocation,
            major=VERSION[0],
            minor=VERSION[1],
            micro=VERSION[2]);
        return;
    }

    // First map all paths to absolute paths
    let iter = matches.free.iter().map(|p| {
        let rel = Path::new(p.as_slice());
        os::make_absolute(&rel)
    });

    // If a symlink follow is desired, map it
    if !matches.opt_present("s") {
        let sym_iter = iter.map(|p| {
            match realpath::eval_symlinks(&p, MAX_DEPTH) {
                Ok(r) => { r },
                Err(e) => { panic!(format!("{}", e)) }
            }
        });
        print_results(sym_iter);
    } else {
        print_results(iter);
    }
}
