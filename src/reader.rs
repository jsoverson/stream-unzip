use bytes::{Buf, Bytes, BytesMut};

static LOCAL_FILE_HEADER: [u8; 4] = [b'P', b'K', 0x03, 0x04];
static COMPRESSION_NONE: [u8; 2] = [0x00, 0x00];
static COMPRESSION_DEFLATE: [u8; 2] = [0x08, 0x00];
static ZIP64_SIZE: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

#[derive(Default, Debug)]
pub struct ZipReader {
    curr_entry: Option<ZipEntry>,
    buffer: BytesMut,
    entries: Vec<ZipEntry>,
}

impl ZipReader {
    pub fn update(&mut self, bytes: Vec<u8>) {
        self.buffer.extend(bytes);
        self.process_buffer();
    }

    pub fn finish(&mut self) {
        self.process_buffer();
        if let Some(curr_entry) = self.curr_entry.take() {
            self.entries.push(curr_entry);
        }
    }

    pub fn entries(&self) -> &[ZipEntry] {
        &self.entries
    }

    pub fn into_entries(self) -> Vec<ZipEntry> {
        self.entries
    }

    pub fn flush(&mut self) {
        self.entries.clear();
    }

    fn process_buffer(&mut self) {
        let mut i = 0;
        let mut last_file_offset = 0;

        while i < self.buffer.len() {
            let mut header = [0; 4];
            if i + 4 > self.buffer.len() {
                break;
            }

            header.copy_from_slice(&self.buffer[i..i + 4]);
            if header == LOCAL_FILE_HEADER {
                let last = self.curr_entry.replace(ZipEntry::default());
                if let Some(mut curr_entry) = last {
                    curr_entry.bytes.extend(&self.buffer[..i]);
                    self.entries.push(curr_entry);
                }
                i += 4;
                last_file_offset = i;
                continue;
            } else {
                i += 1;
                continue;
            }
        }

        if let Some(curr_entry) = self.curr_entry.as_mut() {
            curr_entry.bytes.extend(&self.buffer[last_file_offset..]);
        }

        self.buffer.clear();
    }
}

#[derive(Debug, Default)]
pub struct ZipEntry {
    bytes: BytesMut,
    name: String,
}

impl ZipEntry {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bytes(&self) -> &BytesMut {
        &self.bytes
    }

    pub fn deflate(mut self) -> DeflatedEntry {
        let _version = self.bytes.split_to(2);
        let _gp_bit_flag = self.bytes.split_to(2);
        let _compression_method = self.bytes.split_to(2);
        let _last_mod_time = self.bytes.split_to(2);
        let _last_mod_date = self.bytes.split_to(2);
        let _crc = self.bytes.split_to(4);
        let compressed_size = self.bytes.split_to(4);
        let compressed_size = u32::from_le_bytes([
            compressed_size[0],
            compressed_size[1],
            compressed_size[2],
            compressed_size[3],
        ]) as _;
        let uncompressed_size = self.bytes.split_to(4);
        let uncompressed_size = u32::from_le_bytes([
            uncompressed_size[0],
            uncompressed_size[1],
            uncompressed_size[2],
            uncompressed_size[3],
        ]) as _;
        let filename_len = self.bytes.split_to(2);
        let _extra_field_len = self.bytes.split_to(2);
        let filename = self
            .bytes
            .split_to(u16::from_le_bytes([filename_len[0], filename_len[1]]) as _);

        let _extra_field = self
            .bytes
            .split_to(u16::from_le_bytes([_extra_field_len[0], _extra_field_len[1]]) as _);
        let bytes = self.bytes.split_to(compressed_size);
        DeflatedEntry {
            filename: String::from_utf8(filename.to_vec()).unwrap(),
            bytes: inflate::inflate_bytes(&bytes).unwrap(),
            compressed_size: compressed_size as _,
            uncompressed_size,
        }
    }
}

pub struct DeflatedEntry {
    pub filename: String,
    pub bytes: Vec<u8>,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
}
