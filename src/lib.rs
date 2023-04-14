mod error;
pub use self::error::*;
mod reader;
pub use reader::*;
mod iterator;
pub use iterator::*;

#[cfg(test)]
mod tests {
    use tokio::io::AsyncReadExt;

    use std::{
        fs::File,
        io::Read,
        path::{Path, PathBuf},
    };

    use super::*;

    async fn test_zip(path: &Path, buffer_size: usize) -> anyhow::Result<()> {
        println!("Buffer size: {}", buffer_size);
        let f = std::fs::File::open(path).unwrap();
        let f2 = std::fs::File::open(path).unwrap();
        let mut expected = zip::ZipArchive::new(f2).unwrap();
        let mut f = tokio::fs::File::from_std(f);
        let mut buff: [u8; 10000] = [0; 10000];
        let mut zip_reader = ZipReader::default();
        while let Ok(num) = f.read(&mut buff).await {
            if num == 0 {
                break;
            }
            let mut left_to_read = num;
            let mut last = 0;
            while left_to_read > 0 {
                let to_read = if left_to_read <= buffer_size {
                    left_to_read
                } else {
                    buffer_size
                };
                zip_reader.update(buff[last..(last + to_read)].to_vec().into());
                last += to_read;
                left_to_read -= to_read;
            }
        }
        zip_reader.finish();
        println!("found {} zip entries", zip_reader.entries().len());
        let expanded = zip_reader
            .drain_entries()
            .into_iter()
            .map(|e| e.inflate())
            .collect::<Vec<_>>();

        assert!(!expanded.is_empty());
        assert_eq!(expanded.len(), expected.len());
        for entry in expanded {
            let entry = entry?;
            // println!("File: {:?}", entry.name());
            // println!("File size: {:?}", entry.uncompressed_size());
            // println!("File compressed size: {:?}", entry.compressed_size());
            let mut expected_entry = expected.by_name(entry.name()).unwrap();
            assert_eq!(expected_entry.size(), entry.uncompressed_size() as _);
            let mut expected_bytes = Vec::new();
            expected_bytes.resize(expected_entry.size() as _, 0);
            expected_entry.read_exact(&mut expected_bytes).unwrap();
            assert_eq!(expected_bytes, entry.data().to_vec());
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
            test_zip(&file.path(), 10).await?;
            test_zip(&file.path(), 300).await?;
            test_zip(&file.path(), 1024).await?;
            test_zip(&file.path(), 1024 * 1024).await?;
        }
        Ok(())
    }

    #[test]
    fn test_iter() {
        let file = std::fs::File::open(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/readme.zip"),
        )
        .unwrap();
        let zip: ZipIterator<File, 32> = file.into();

        let mut entries = Vec::new();
        for entry in zip {
            entries.push(entry);
        }

        assert_eq!(1, entries.len());
    }
}
