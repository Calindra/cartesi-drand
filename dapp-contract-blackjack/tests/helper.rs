use std::{ffi::OsStr, fs};

pub fn clean_files() {
    for path in fs::read_dir("./data/tables").unwrap() {
        let path = path.unwrap().path();
        let extension = path.extension();
        if extension.is_some() && extension.unwrap() == OsStr::new("json") {
            fs::remove_file(path).unwrap();
        }
    }
}
