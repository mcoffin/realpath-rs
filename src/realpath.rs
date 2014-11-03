use std::io;
use std::path::Path;

pub fn eval_symlinks(path: &Path, max_depth: uint) -> io::IoResult<Path> {
    let mut ret = path.root_path().expect("no root path found");
    for comp in path.components() {
        ret.push(comp);

        let mut link_count = 0;
        loop {
            if link_count >= max_depth {
                return Err(io::standard_error(io::InvalidInput));
            }

            match io::fs::lstat(&ret) {
                Err(e) => { return Err(e) },
                Ok(ref stats) => match stats.kind {
                    io::TypeSymlink => {
                        link_count += 1;
                        let next = try!(io::fs::readlink(&ret));
                        ret.pop();
                        ret.push(next);
                    },
                    _ => break
                }
            }
        }
    }
    Ok(ret)
}
