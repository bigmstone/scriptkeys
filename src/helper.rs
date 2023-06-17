use std::path::{Path, PathBuf};

pub fn parse_path(path: &str) -> PathBuf {
    let base = directories::BaseDirs::new().unwrap();
    let path = path.replace("$HOME", base.home_dir().to_str().unwrap());

    Path::new(&path).to_owned()
}
