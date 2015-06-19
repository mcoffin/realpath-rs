extern crate getopts;

use getopts::Options;
use std::env;
use std::io;
use std::fs;
use std::path::{Path, PathBuf, Component};

const MAX_DEPTH: usize = 256;

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    let invocation = args[0].clone();

    // Create and parse command line arguments
    let opts = create_options();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!(e),
    };

    // If help option present, print help and exit
    if matches.opt_present("h") {
        print_usage(&invocation, &opts);
        return;
    }

    // If version option present, print version and exit
    if matches.opt_present("v") {
        print_version(&invocation);
        return;
    }

    // Make all paths absolute
    let iter = matches.free.iter().map(|p| make_absolute(p));

    let zero_opt = matches.opt_present("z");

    // If a symlink follow is desired, map it.
    if !matches.opt_present("s") {
        let iter = iter.map(|p| {
            match read_all_links(p, MAX_DEPTH) {
                Ok(r) => r,
                Err(e) => panic!(format!("{}", e)),
            }
        });
        print_results(iter, zero_opt);
    } else {
        print_results(iter, zero_opt);
    }
}

fn make_absolute<P: AsRef<Path>>(path: P) -> PathBuf {
    // If not an absolute path, append to the current dir
    let path = path.as_ref();
    let path = if path.is_absolute() {
        PathBuf::from(path)
    } else {
        let current = match env::current_dir() {
            Ok(d) => d,
            Err(e) => panic!(e),
        };
        current.join(path)
    };

    // Now strip all .. and . references
    strip_rel(path)
}

fn strip_rel<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let mut reassembled = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {},
            Component::ParentDir => {
                reassembled.pop();
            },
            x => reassembled.push(x.as_os_str()),
        }
    }
    reassembled
}

fn read_all_links<P: AsRef<Path>>(path: P, max_depth: usize) -> io::Result<PathBuf> {
    let path = path.as_ref();

    let mut ret = PathBuf::from(Component::RootDir.as_os_str());
    
    for comp in path.components() {
        match comp {
            Component::Normal(c) => {
                let mut link_count = 0;
                let mut test = ret.join(c);
                loop {
                    if link_count >= max_depth {
                        break;
                    }
                    match fs::read_link(&test) {
                        Ok(new_path) => {
                            link_count += 1;
                            test = strip_rel(ret.join(new_path));
                        },
                        _ => break,
                    }
                }
                ret.push(test);
            },
            c => ret.push(c.as_os_str()),
        }
    }

    Ok(ret)
}

fn print_results<P: AsRef<Path>, I: Iterator<Item=P>>(iter: I, zero_sep: bool) {
    for path in iter {
        let path = path.as_ref();
        if zero_sep {
            print!("{}\0", path.display());
        } else {
            println!("{}", path.display());
        }
    }
}

fn print_version<T: AsRef<str>>(invocation: T) {
    println!("{name} v{major}.{minor}.{patch}",
             name = invocation.as_ref(),
             major = env!("CARGO_PKG_VERSION_MAJOR"),
             minor = env!("CARGO_PKG_VERSION_MINOR"),
             patch = env!("CARGO_PKG_VERSION_PATCH"));
}

fn print_usage<T: AsRef<str>>(invocation: T, opts: &Options) {
    let short = format!("{} path...", opts.short_usage(invocation.as_ref()));
    println!("{}", opts.usage(short.as_ref()));
}

fn create_options() -> Options {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print usage info then exit");
    opts.optflag("v", "version", "print version info then exit");
    opts.optflag("s", "strip", "don't expand symlinks");
    opts.optflag("z", "zero", "print zero separators instead of newlines");
    opts
}
