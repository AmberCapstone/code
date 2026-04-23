use proto::sensor::fpga::flash::Segment;

pub const PAGE_SIZE: usize = 0x100;
pub const SUBSECTOR_SIZE: usize = 0x1000;
pub const SECTOR_SIZE: usize = 0x1_0000;
pub const CHIP_SIZE: usize = 0x10_0000;

#[derive(Debug)]
pub struct Region {
    start: usize, // inclusive
    length: usize,
}

impl Region {
    pub const fn subsectors(start: usize, count: usize) -> Self {
        Self {
            start: start * SUBSECTOR_SIZE,
            length: count * SUBSECTOR_SIZE,
        }
    }

    pub const fn start(&self) -> usize {
        self.start
    }

    pub const fn length(&self) -> usize {
        self.length
    }

    pub const fn end(&self) -> usize {
        self.start + self.length
    }
}

pub const fn get_region(segment: Segment) -> Region {
    use Segment::*;
    match segment {
        Unknown => Region::subsectors(0, 0),
        Fpga => Region::subsectors(0, 32),
        Qvga0 => Region::subsectors(32, 40),
        Qvga1 => Region::subsectors(72, 40),
        Qvga2 => Region::subsectors(112, 40),
        User => Region::subsectors(152, 40),
        Dfu => Region::subsectors(192, 64),
    }
}

/// Compile time layout check.
const fn validate_layout() {
    use Segment::*;

    // EnumIter is not const so this list but be manually updated.
    let segments = &[Unknown, Fpga, Qvga0, Qvga1, Qvga2, User, Dfu];

    let mut i = 0;
    let mut total_size = 0;

    while i < segments.len() {
        let r = get_region(segments[i]);
        assert!(r.start.is_multiple_of(SUBSECTOR_SIZE));
        assert!(r.length.is_multiple_of(SUBSECTOR_SIZE));

        total_size += r.length;

        assert!(r.end() <= CHIP_SIZE, "Region doesn't fit on chip");

        // check region doesn't overlap with any others
        let mut j = i + 1;
        while j < segments.len() {
            let reg_j = get_region(segments[j]);
            assert!((r.end() <= reg_j.start) || (reg_j.end() <= r.start), "Regions overlap");

            j += 1;
        }

        i += 1;
    }

    assert!(total_size == CHIP_SIZE, "Not all space is used");
}

const _: () = validate_layout();
