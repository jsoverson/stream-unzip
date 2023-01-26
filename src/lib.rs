mod error;
pub use self::error::*;
mod reader;
pub use reader::*;

#[cfg(test)]
mod tests {
    use tokio::io::AsyncReadExt;

    use std::{
        io::Read,
        path::{Path, PathBuf},
    };

    use super::*;

    async fn test_zip(path: &Path) -> anyhow::Result<()> {
        let f = std::fs::File::open(path).unwrap();
        let f2 = std::fs::File::open(path).unwrap();
        let mut expected = zip::ZipArchive::new(f2).unwrap();
        let mut f = tokio::fs::File::from_std(f);
        let mut buff: [u8; 300] = [0; 300];
        let mut zip_reader = ZipReader::default();
        while let Ok(num) = f.read(&mut buff).await {
            if num == 0 {
                break;
            }
            zip_reader.update(buff[..num].to_vec());
        }
        zip_reader.finish();
        println!("found {} zip entries", zip_reader.entries().len());
        let expanded = zip_reader
            .into_entries()
            .into_iter()
            .map(|e| e.deflate())
            .collect::<Vec<_>>();

        assert!(!expanded.is_empty());
        for entry in expanded {
            let entry = entry?;
            println!("File: {:?}", entry.filename);
            println!("File size: {:?}", entry.uncompressed_size);
            println!("File compressed size: {:?}", entry.compressed_size);
            let mut expected_entry = expected.by_name(&entry.filename).unwrap();
            assert_eq!(expected_entry.size(), entry.uncompressed_size as _);
            let mut expected_bytes = Vec::new();
            expected_bytes.resize(expected_entry.size() as _, 0);
            expected_entry.read_exact(&mut expected_bytes).unwrap();
            assert_eq!(expected_bytes, entry.bytes.to_vec());
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_async() -> anyhow::Result<()> {
        let mut files =
            std::fs::read_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata")).unwrap();

        while let Some(Ok(file)) = files.next() {
            if file.path().extension() != Some("zip".as_ref()) {
                println!("    skipping {}", file.path().to_string_lossy());
                continue;
            }
            println!("--> testing {}", file.path().to_string_lossy());
            test_zip(&file.path()).await?;
        }
        Ok(())
    }
}
