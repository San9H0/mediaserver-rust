use std::fs;
use std::path::Path;

pub fn create_directory_if_not_exists(file_path: &str) -> std::io::Result<()> {
    let path = Path::new(file_path);

    // 부모 디렉터리 경로 추출
    if let Some(parent) = path.parent() {
        // 디렉터리가 존재하지 않으면 생성
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    Ok(())
}