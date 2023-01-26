mod error;
pub use self::error::*;
mod reader;
pub use reader::*;

#[cfg(test)]
mod tests {
    use tokio::io::AsyncReadExt;

    // use super::{encoding::Encoding, prelude::*, Archive, Error};

    use std::{
        collections::HashMap,
        io::Read,
        path::{Path, PathBuf},
    };

    use super::*;

    enum ZipSource {
        File(&'static str),
        Func(&'static str, Box<dyn Fn() -> Vec<u8>>),
    }

    struct ZipTest {
        source: ZipSource,
        // expected_encoding: Option<Encoding>,
        comment: Option<&'static str>,
        files: Vec<ZipTestFile>,
        error: Option<super::Error>,
    }

    impl Default for ZipTest {
        fn default() -> Self {
            Self {
                source: ZipSource::Func("default.zip", Box::new(|| unreachable!())),
                // expected_encoding: None,
                comment: None,
                files: vec![],
                error: None,
            }
        }
    }

    #[derive(Debug)]
    struct ZipTestFile {
        name: &'static str,
        mode: Option<u32>,
        content: FileContent,
    }

    #[derive(Debug)]
    enum FileContent {
        Unchecked,
        Bytes(Vec<u8>),
        File(&'static str),
    }

    impl Default for ZipTestFile {
        fn default() -> Self {
            Self {
                name: "default",
                mode: None,
                content: FileContent::Unchecked,
            }
        }
    }

    impl ZipTest {
        fn name(&self) -> &'static str {
            match &self.source {
                ZipSource::File(name) => name,
                ZipSource::Func(name, _f) => name,
            }
        }

        fn bytes(&self) -> Vec<u8> {
            match &self.source {
                ZipSource::File(name) => {
                    let path = {
                        let zips_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata");
                        zips_dir.join(name)
                    };
                    std::fs::read(path).unwrap()
                }
                ZipSource::Func(_name, f) => f(),
            }
        }
    }

    async fn test_zip(path: &Path) {
        let f = std::fs::File::open(path).unwrap();
        let f2 = std::fs::File::open(path).unwrap();
        let mut expected = zip::ZipArchive::new(f2).unwrap();
        let mut f = tokio::fs::File::from_std(f);
        let mut buff: [u8; 1000] = [0; 1000];
        let mut zip_reader = ZipReader::default();
        while let Ok(num) = f.read(&mut buff).await {
            if num == 0 {
                break;
            }
            zip_reader.update(buff.to_vec());
            for entry in zip_reader.entries() {
                println!("File: {:?}", entry.name());
            }
        }
        zip_reader.finish();
        println!("found {} zip entries", zip_reader.entries().len());
        let expanded = zip_reader
            .into_entries()
            .into_iter()
            .map(|e| e.deflate())
            .collect::<Vec<_>>();

        for entry in expanded {
            // println!("File: {:?}", entry.filename);
            // println!("File size: {:?}", entry.uncompressed_size);
            // println!("File compressed size: {:?}", entry.compressed_size);
            let mut expected_entry = expected.by_name(&entry.filename).unwrap();
            assert_eq!(expected_entry.size(), entry.uncompressed_size);
            let mut expected_bytes = Vec::new();
            expected_bytes.resize(expected_entry.size() as _, 0);
            expected_entry.read_exact(&mut expected_bytes).unwrap();
            assert_eq!(expected_bytes, entry.bytes.to_vec());
        }
    }

    #[tokio::test]
    async fn test_async() {
        let mut files =
            std::fs::read_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata")).unwrap();

        while let Some(Ok(file)) = files.next() {
            println!("testing {}", file.path().to_string_lossy());
            test_zip(&file.path()).await;
        }
    }
}
