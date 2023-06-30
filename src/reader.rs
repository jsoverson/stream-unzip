use std::collections::VecDeque;

use bytes::{Buf, Bytes, BytesMut};

static H_LOCAL_FILE: [u8; 4] = [b'P', b'K', 0x03, 0x04];
static H_DATA_DESCRIPTOR: [u8; 4] = [b'P', b'K', 0x07, 0x08];
static H_CENTRAL_DIRECTORY: [u8; 4] = [b'P', b'K', 0x01, 0x02];
static H_EO_CENTRAL_DIRECTORY: [u8; 4] = [b'P', b'K', 0x05, 0x06];

// Unused but will be needed
// static COMPRESSION_NONE: [u8; 2] = [0x00, 0x00];
// static COMPRESSION_DEFLATE: [u8; 2] = [0x08, 0x00];
// static ZIP64_SIZE: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

#[derive(Debug, Clone)]
enum Header {
    LocalFile(LocalFileHeader),
    DataDescriptor(DataDescriptor),
    CentralDirectory(CentralDirectoryHeader),
    EndOfCentralDirectory(EndOfCentralDirectory),
}

impl Header {}

#[derive(Debug, Clone)]
pub struct LocalFileHeader {
    pub version: u16,
    pub flags: u16,
    pub compression: u16,
    pub last_mod_time: u16,
    pub last_mod_date: u16,
    pub crc32: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub file_name_length: u16,
    pub extra_field_length: u16,
    pub filename: String,
    pub extra_field: Vec<u8>,
}

impl LocalFileHeader {
    fn size() -> usize {
        26
    }
}

#[derive(Debug, Clone)]
pub struct DataDescriptor {
    pub crc32: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
}

impl DataDescriptor {
    fn size() -> usize {
        12
    }
}

#[derive(Debug, Clone)]
pub struct CentralDirectoryHeader {
    pub version_made_by: u16,
    pub version_needed_to_extract: u16,
    pub flags: u16,
    pub compression: u16,
    pub last_mod_time: u16,
    pub last_mod_date: u16,
    pub crc32: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub file_name_length: u16,
    pub extra_field_length: u16,
    pub file_comment_length: u16,
    pub disk_number_start: u16,
    pub internal_file_attributes: u16,
    pub external_file_attributes: u32,
    pub relative_offset_of_local_header: u32,
}
impl CentralDirectoryHeader {
    fn size() -> usize {
        42
    }
}
#[derive(Debug, Clone)]
pub struct EndOfCentralDirectory {
    pub disk_number: u16,
    pub disk_with_central_directory: u16,
    pub number_of_entries_on_disk: u16,
    pub number_of_entries: u16,
    pub size_of_central_directory: u32,
    pub offset_of_start_of_central_directory: u32,
    pub zip_file_comment_length: u16,
}

impl EndOfCentralDirectory {
    fn size() -> usize {
        16
    }
}

fn decode_header(b: &mut BytesMut) -> Option<Header> {
    if b.remaining() < 4 {
        return None;
    }
    let header = &b[0..4];

    if header == H_LOCAL_FILE {
        let base_size = LocalFileHeader::size() + 4;
        if b.remaining() < base_size {
            return None;
        }
        let mut intermediate: BytesMut = BytesMut::zeroed(base_size);
        intermediate.copy_from_slice(&b[0..base_size]);
        intermediate.advance(4);
        let version = intermediate.get_u16_le();
        let flags = intermediate.get_u16_le();
        let compression = intermediate.get_u16_le();
        let last_mod_time = intermediate.get_u16_le();
        let last_mod_date = intermediate.get_u16_le();
        let crc32 = intermediate.get_u32_le();
        let compressed_size = intermediate.get_u32_le();
        let uncompressed_size = intermediate.get_u32_le();
        let file_name_length = intermediate.get_u16_le();
        let extra_field_length = intermediate.get_u16_le();
        if (b.remaining() - base_size) < (file_name_length + extra_field_length) as usize {
            return None;
        } else {
            b.advance(base_size);
        }

        let filename = String::from_utf8(b.split_to(file_name_length as usize).to_vec()).unwrap();
        let extra_field = b.split_to(extra_field_length as usize).to_vec();
        let h = Header::LocalFile(LocalFileHeader {
            version,
            flags,
            compression,
            last_mod_time,
            last_mod_date,
            crc32,
            compressed_size,
            uncompressed_size,
            file_name_length,
            extra_field_length,
            filename,
            extra_field,
        });
        Some(h)
    } else if header == H_DATA_DESCRIPTOR {
        if b.remaining() < DataDescriptor::size() + 4 {
            return None;
        }
        b.advance(4);
        let crc32 = b.get_u32_le();
        let compressed_size = b.get_u32_le();
        let uncompressed_size = b.get_u32_le();
        let h = Header::DataDescriptor(DataDescriptor {
            crc32,
            compressed_size,
            uncompressed_size,
        });
        Some(h)
    } else if header == H_EO_CENTRAL_DIRECTORY {
        if b.remaining() < EndOfCentralDirectory::size() + 4 {
            return None;
        }
        b.advance(4);
        let disk_number = b.get_u16_le();
        let disk_with_central_directory = b.get_u16_le();
        let number_of_entries_on_disk = b.get_u16_le();
        let number_of_entries = b.get_u16_le();
        let size_of_central_directory = b.get_u32_le();
        let offset_of_start_of_central_directory = b.get_u32_le();
        let zip_file_comment_length = b.get_u16_le();
        let h = Header::EndOfCentralDirectory(EndOfCentralDirectory {
            disk_number,
            disk_with_central_directory,
            number_of_entries_on_disk,
            number_of_entries,
            size_of_central_directory,
            offset_of_start_of_central_directory,
            zip_file_comment_length,
        });
        Some(h)
    } else if header == H_CENTRAL_DIRECTORY {
        if b.remaining() < CentralDirectoryHeader::size() + 4 {
            return None;
        }
        b.advance(4);
        let version_made_by = b.get_u16_le();
        let version_needed_to_extract = b.get_u16_le();
        let flags = b.get_u16_le();
        let compression = b.get_u16_le();
        let last_mod_time = b.get_u16_le();
        let last_mod_date = b.get_u16_le();
        let crc32 = b.get_u32_le();
        let compressed_size = b.get_u32_le();
        let uncompressed_size = b.get_u32_le();
        let file_name_length = b.get_u16_le();
        let extra_field_length = b.get_u16_le();
        let file_comment_length = b.get_u16_le();
        let disk_number_start = b.get_u16_le();
        let internal_file_attributes = b.get_u16_le();
        let external_file_attributes = b.get_u32_le();
        let relative_offset_of_local_header = b.get_u32_le();
        let h = Header::CentralDirectory(CentralDirectoryHeader {
            version_made_by,
            version_needed_to_extract,
            flags,
            compression,
            last_mod_time,
            last_mod_date,
            crc32,
            compressed_size,
            uncompressed_size,
            file_name_length,
            extra_field_length,
            file_comment_length,
            disk_number_start,
            internal_file_attributes,
            external_file_attributes,
            relative_offset_of_local_header,
        });
        Some(h)
    } else {
        None
    }
}

#[derive(Default, Debug)]
pub struct ZipReader {
    curr_entry: Option<ZipEntry>,
    buffer: BytesMut,
    entries: VecDeque<ZipEntry>,
}

impl ZipReader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, bytes: Bytes) {
        self.buffer.extend(bytes);
        self.process_buffer();
    }

    pub fn finish(&mut self) {
        self.process_buffer();
        if let Some(curr_entry) = self.curr_entry.take() {
            self.entries.push_back(curr_entry);
        }
    }

    pub fn entries(&mut self) -> &[ZipEntry] {
        self.entries.make_contiguous();
        &self.entries.as_slices().0
    }

    pub fn take_entry(&mut self) -> Option<ZipEntry> {
        self.entries.pop_front()
    }

    pub fn drain_entries(&mut self) -> Vec<ZipEntry> {
        self.entries.drain(0..).collect()
    }

    pub fn flush(&mut self) {
        self.entries.clear();
    }

    fn process_buffer(&mut self) {
        let mut i = 0;

        while i < self.buffer.len() {
            if let Some(header) = decode_header(&mut self.buffer) {
                if let Some(curr_entry) = self.curr_entry.take() {
                    self.entries.push_back(curr_entry);
                }
                if let Header::LocalFile(local) = &header {
                    let mut new_entry = ZipEntry::new(local.clone());
                    let copy = if local.compressed_size as usize > self.buffer.remaining() {
                        self.buffer.remaining()
                    } else {
                        local.compressed_size as usize
                    };
                    new_entry.bytes.extend(self.buffer.copy_to_bytes(copy));
                    self.curr_entry = Some(new_entry);
                    continue;
                }
                if let Header::DataDescriptor(data) = &header {
                    if let Some(curr_entry) = self.curr_entry.as_mut() {
                        curr_entry.header.crc32 = data.crc32;
                        curr_entry.header.compressed_size = data.compressed_size;
                        curr_entry.header.uncompressed_size = data.uncompressed_size;
                    }
                }
                i = self.buffer.len() - self.buffer.remaining();
                continue;
            } else {
                i += 1;
                continue;
            }
        }

        if let Some(curr_entry) = self.curr_entry.as_mut() {
            let mut remaining = curr_entry.header.compressed_size as usize - curr_entry.bytes.len();
            if remaining > self.buffer.remaining() {
                remaining = self.buffer.remaining();
            }

            if remaining > 0 {
                curr_entry.bytes.extend(&self.buffer.split_to(remaining));
            }
        }
    }
}

#[derive(Debug)]
pub struct ZipEntry {
    header: LocalFileHeader,
    bytes: BytesMut,
    name: String,
}

impl ZipEntry {
    pub fn new(header: LocalFileHeader) -> Self {
        Self {
            bytes: BytesMut::with_capacity(header.compressed_size as usize),
            header,
            name: String::new(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn compressed_data(&self) -> &BytesMut {
        &self.bytes
    }

    pub fn header(&self) -> &LocalFileHeader {
        &self.header
    }

    pub fn inflate(self) -> Result<DeflatedEntry, crate::Error> {
        let bytes = self.bytes;

        Ok(DeflatedEntry {
            bytes: inflate::inflate_bytes(&bytes).unwrap().into(),
            header: self.header,
        })
    }
}

/// An extracted entry from a zip file.
pub struct DeflatedEntry {
    header: LocalFileHeader,
    bytes: Bytes,
}

impl DeflatedEntry {
    /// Returns the header and the decompressed data.
    pub fn into_parts(self) -> (LocalFileHeader, Bytes) {
        (self.header, self.bytes)
    }

    /// Returns a reference to the decompressed data.
    pub fn data(&self) -> &Bytes {
        &self.bytes
    }

    /// Returns the filename of the zip entry.
    pub fn name(&self) -> &str {
        &self.header.filename
    }

    /// Returns the compressed size of the data.
    pub fn compressed_size(&self) -> u32 {
        self.header.compressed_size
    }

    /// Returns the uncompressed size of the data.
    pub fn uncompressed_size(&self) -> u32 {
        self.header.uncompressed_size
    }
}

impl From<DeflatedEntry> for Bytes {
    fn from(entry: DeflatedEntry) -> Self {
        entry.bytes
    }
}
