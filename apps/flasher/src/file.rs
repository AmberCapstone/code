use flash_layout::PAGE_SIZE;
use proto::sensor::fpga::flash::{Page, Segment};
use std::{fs::File, io::Read, iter::Enumerate, path::Path, slice::ChunksExact};

pub struct FlashFile {
    bytes: Vec<u8>,
    segment: Segment,
}

#[derive(Debug)]
pub enum Error {
    Os(std::io::Error),
    TooLarge { length: usize, segment_size: usize },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Os(error) => write!(f, "OS Error: {error}"),
            Error::TooLarge { length, segment_size } => {
                write!(f, "File doesn't fit in segment ({length}/{segment_size} bytes)")
            }
        }
    }
}

impl std::error::Error for Error {}

impl FlashFile {
    pub fn new(path: &Path, segment: Segment, pad: u8) -> Result<Self, Error> {
        let mut file = File::open(path).map_err(Error::Os)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(Error::Os)?;

        let length = bytes.len();
        let segment_size = flash_layout::get_region(segment).length();
        if length > segment_size {
            return Err(Error::TooLarge { length, segment_size });
        }

        if length < segment_size {
            println!("Padding file from {length} bytes to {segment_size} with 0x{pad:02x}");
            bytes.resize_with(segment_size, || pad);
        }

        Ok(FlashFile { bytes, segment })
    }

    pub fn size(&self) -> usize {
        self.bytes.len()
    }

    pub fn num_pages(&self) -> usize {
        self.size() / PAGE_SIZE
    }

    pub fn pages(&self) -> PageIter<'_> {
        assert!(self.size().is_multiple_of(PAGE_SIZE));
        PageIter(self.bytes.chunks_exact(PAGE_SIZE).enumerate())
    }

    pub fn segment(&self) -> Segment {
        self.segment
    }
}

pub struct PageIter<'a>(Enumerate<ChunksExact<'a, u8>>);

impl Iterator for PageIter<'_> {
    type Item = Page;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(pg_num, data)| {
            #[allow(clippy::cast_possible_truncation, reason = "pg_num is small")]
            let pg_num = pg_num as u32;

            let pg_crc = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
            let mut digest = pg_crc.digest();

            digest.update(&pg_num.to_le_bytes());
            digest.update(data);
            let crc = digest.finalize();

            Page {
                page_number: Some(pg_num),
                data: Some(data.to_vec()),
                crc: Some(crc),
            }
        })
    }
}
