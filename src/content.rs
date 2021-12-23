use std::path::Path;

pub fn get_path(path: &str) -> String {
    let base_dir = if std::env::var("CONTENT_IN_WORKING_DIR") == Ok("1".to_owned()) {
        std::env::current_dir().unwrap()
    } else {
        std::env::current_exe().unwrap().parent().unwrap().to_owned()
    };

    base_dir.join(Path::new(path)).to_str().unwrap().to_owned()
}