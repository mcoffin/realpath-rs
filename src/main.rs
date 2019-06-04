#[macro_use]
extern crate clap;

use std::env;
use std::fs;
use std::io;
use std::path::{self, Path, PathBuf};

const FILE_ARG: &'static str = "file";
const STRIP_ARG: &'static str = "strip";
const ZERO_ARG: &'static str = "zero";

const MAX_DEPTH: usize = 256;

fn application() -> clap::App<'static, 'static> {
    use clap::{App, Arg};
    let strip_arg = Arg::with_name(STRIP_ARG)
        .short("s")
        .long("strip")
        .help("don't expand symlinks");
    let zero_arg = Arg::with_name(ZERO_ARG)
        .short("z")
        .long("zero")
        .help("end each output line with NUL, not newline");
    let file_arg = Arg::with_name(FILE_ARG)
        .takes_value(true)
        .multiple(true)
        .required(true)
        .value_name("FILE");
    App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .arg(strip_arg)
        .arg(zero_arg)
        .arg(file_arg)
}

fn make_absolute<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let path = if path.is_absolute() {
        PathBuf::from(path)
    } else {
        let current = env::current_dir().expect("Invalid current directory");
        current.join(path)
    };

    // Now strip all .. and . references
    strip_rel(path)
}

fn strip_rel<P: AsRef<Path>>(path: P) -> PathBuf {
    use std::path::Component;

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
    use std::path::Component;
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
                    if !fs::symlink_metadata(&test)?.file_type().is_symlink() {
                        break;
                    }
                    match fs::read_link(&test) {
                        Ok(new_path) => {
                            link_count += 1;
                            test = strip_rel(ret.join(new_path));
                        },
                        e => return e,
                    }
                }
                ret.push(test);
            },
            c => ret.push(c.as_os_str()),
        }
    }
    Ok(ret)
}

trait PrintResult {
    fn print_result(v: path::Display);
}

struct NewlineSeparated;

struct ZeroSeparated;

impl PrintResult for NewlineSeparated {
    fn print_result(v: path::Display) {
        println!("{}", v);
    }
}

impl PrintResult for ZeroSeparated {
    fn print_result(v: path::Display) {
        print!("{}\0", v);
    }
}

fn print_results_real<P, I, Printer>(iter: I) where
    P: AsRef<Path>,
    I: Iterator<Item=P>,
    Printer: PrintResult
{
    for path in iter {
        let path = path.as_ref();
        Printer::print_result(path.display());
    }
}

#[inline(always)]
fn print_results<P, I>(iter: I, print_zero: bool) where
    P: AsRef<Path>,
    I: Iterator<Item=P>,
{
    if print_zero {
        print_results_real::<P, I, ZeroSeparated>(iter);
    } else {
        print_results_real::<P, I, NewlineSeparated>(iter);
    }
}

fn main() {
    let matches = application().get_matches();
    let iter = matches.values_of_os(FILE_ARG).unwrap().map(make_absolute);
    let print_zero = matches.is_present(ZERO_ARG);
    if matches.is_present(STRIP_ARG) {
        print_results(iter, print_zero);
    } else {
        let iter = iter.map(|p| {
            read_all_links(p, MAX_DEPTH)
                .unwrap_or_else(|e| panic!(format!("{}", e)))
        });
        print_results(iter, print_zero);
    }
}
