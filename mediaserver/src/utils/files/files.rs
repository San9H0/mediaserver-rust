use std::path::Path;

use tokio::io::AsyncWriteExt;

pub async fn write_file_force<'a>(p: &str, data: &'a [u8]) -> anyhow::Result<()> {
    let path: &Path = Path::new(p);
    if let Some(parent) = path.parent() {
        // 부모 디렉토리가 있는지 확인하고 없으면 생성
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::File::create(path).await?.write_all(data).await?;
    Ok(())
}
