use crate::{ZipEntry, ZipReader};

struct ZipIterator<F, const N: usize> {
    file: F,
    zip_reader: ZipReader,
}

impl<F, const N: usize> ZipIterator<F, N> {
    pub fn new(file: F) -> Self {
        Self {
            file,
            zip_reader: ZipReader::default(),
        }
    }
}

impl<F, const N: usize> From<F> for ZipIterator<F, N>
where
    F: std::io::Read,
{
    fn from(value: F) -> Self {
        ZipIterator::new(value)
    }
}

impl<F, const N: usize> Iterator for ZipIterator<F, N>
where
    F: std::io::Read,
{
    type Item = ZipEntry;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.zip_reader.take_entry() {
                None => {
                    let mut buf = [0u8; N];
                    let num = self.file.read(&mut buf).unwrap();

                    if num == 0 {
                        return None;
                    }

                    self.zip_reader.update(buf[..num].to_vec().into());
                }
                entry => return entry,
            }
        }
    }
}
