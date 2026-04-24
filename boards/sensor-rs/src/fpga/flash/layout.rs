use super::spiflash::size;
use crate::proto::sensor_::fpga_::flash_::Segment;

pub(super) struct Bounds {
    pub origin: u32,
    pub length: u32,
}

impl Bounds {
    fn from_subsectors(origin: u32, length: u32) -> Self {
        Self {
            origin: origin * size::SUBSECTOR,
            length: length * size::SUBSECTOR,
        }
    }

    pub fn end(&self) -> u32 {
        self.origin + self.length
    }

    pub fn num_pages(&self) -> u32 {
        self.length / size::PAGE
    }

    pub fn num_subsectors(&self) -> u32 {
        self.length / size::SUBSECTOR
    }
}

impl defmt::Format for Bounds {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ORIGIN={}, LENGTH={}", self.origin, self.length)
    }
}

pub(super) fn get_bounds(segment: Segment) -> Bounds {
    // The developer is resonsible for ensuring these bounds do not overlap
    // The W
    match segment {
        Segment::Fpga => Bounds::from_subsectors(0, 32),
        Segment::Qvga0 => Bounds::from_subsectors(32, 40),
        Segment::Qvga1 => Bounds::from_subsectors(72, 40),
        Segment::Qvga2 => Bounds::from_subsectors(112, 40),
        Segment::User => Bounds::from_subsectors(152, 40),
        Segment::Dfu => Bounds::from_subsectors(192, 64),
        _ => Bounds::from_subsectors(0, 0),
    }
}
