#![allow(clippy::doc_markdown)]

use strum::FromRepr;

#[repr(u8)]
#[derive(Clone, Copy, FromRepr, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(std::hash::Hash))]
#[allow(non_camel_case_types, clippy::upper_case_acronyms, reason = "to match datasheet")]
pub enum Reg {
    /// Gain control
    GAIN = 0x00,
    /// Blue channel gain
    BLUE = 0x01,
    /// Red channel gain
    RED = 0x02,
    /// Vertical Frame Control
    VREF = 0x03,
    /// Common Control 1
    COM1 = 0x04,
    /// U/B Average Level
    BAVE = 0x05,
    /// Y/Gb Average Level
    GbAVE = 0x06,
    /// Exposure Value
    AECHH = 0x07,
    /// V/R Average Level
    RAVE = 0x08,
    /// Common Control 2
    COM2 = 0x09,
    /// Product ID MSB (read only)
    PID = 0x0A,
    /// Product ID LSB (read only)
    VER = 0x0B,
    /// Common Control 3
    COM3 = 0x0C,
    /// Common Control 4
    COM4 = 0x0D,
    /// Common Control 5
    COM5 = 0x0E,
    /// Common Control 6
    COM6 = 0x0F,
    /// Exposure Value
    AECH = 0x10,
    /// Internal Clock
    CLKRC = 0x11,
    /// Common Control 7
    COM7 = 0x12,
    /// Common Control 8
    COM8 = 0x13,
    /// Common Control 9
    COM9 = 0x14,
    /// Common Control 10
    COM10 = 0x15,
    // Reserved
    _RSVD16 = 0x16,
    /// Output Format - Horizontal Frame Start High
    HSTART = 0x17,
    /// Output Format - Horiztonal Frame End High
    HSTOP = 0x18,
    /// Output Format - Vertical Frame Start High
    VSTRT = 0x19,
    /// Output Format - Vertical Frame End High
    VSTOP = 0x1A,
    /// Data Format - Pixel Delay Select
    PSHFT = 0x1B,
    /// Manufacturer ID High Byte
    MIDH = 0x1C,
    /// Manufacturer ID Low Byte
    MIDL = 0x1D,
    /// Mirror / VFlip Enable
    MVFP = 0x1E,
    /// Reserved
    LAEC = 0x1F,
    /// ADC Control
    ADCCTR0 = 0x20,
    /// Reserved
    ADCCTR1 = 0x21,
    /// Reserved
    ADCCTR2 = 0x22,
    /// Reserved
    ADCCTR3 = 0x23,
    /// AGC / AEC Stable Operating Region Upper Limit
    AEW = 0x24,
    /// AGC / AEC Stable Operating Region Lower Limit
    AEB = 0x25,
    /// AGC / AEC Fast Mode Operating Region
    VPT = 0x26,
    /// B Channel Signal Output Bias
    BBIAS = 0x27,
    /// Gb Channel Signal Output Bias
    GbBIAS = 0x28,
    /// Reserved
    _RSVD29 = 0x29,
    /// Dummy Pixel Insert MSB
    EXHCH = 0x2A,
    /// Dummy Pixel Insert LSB
    EXHCL = 0x2B,
    /// R Channel Output Bias
    RBIAS = 0x2C,
    /// LSB of Insert Dummy Lines in Vertical Direction
    ADVFL = 0x2D,
    /// MSB of Insert Dummy Lines in Vertical Direction
    ADVFH = 0x2E,
    /// Y/G Channel Average Value
    YAVE = 0x2F,
    /// HSYNC Rising Edge Delay
    HSYST = 0x30,
    /// HSYNC Falling Edge Delay
    HSYEN = 0x31,
    /// HREF Control,
    HREF = 0x32,
    /// Reserved
    CHLF = 0x33,
    /// Reserved,
    ARBLM = 0x34,
    /// Reserved
    _RSVD35 = 0x35,
    /// Reserved
    _RSVD36 = 0x36,
    /// Reserved
    ADC = 0x37,
    /// Reserved
    ACOM = 0x38,
    /// Reserved
    OFON = 0x39,
    /// Line Buffer Test Option
    TSLB = 0x3A,
    /// Common Control 11
    COM11 = 0x3B,
    /// Common Control 12
    COM12 = 0x3C,
    /// Common Control 13
    COM13 = 0x3D,
    /// Common Control 14,
    COM14 = 0x3E,
    /// Edge Enhancement Factor
    EDGE = 0x3F,
    /// Common Control 15
    COM15 = 0x40,
    /// Common Control 16
    COM16 = 0x41,
    /// Common Control 17
    COM17 = 0x42,
    /// Reserved
    AWBC1 = 0x43,
    /// Reserved
    AWBC2 = 0x44,
    /// Reserved
    AWBC3 = 0x45,
    /// Reserved
    AWBC4 = 0x46,
    /// Reserved
    AWBC5 = 0x47,
    /// Reserved
    AWBC6 = 0x48,
    /// Reserved
    _RSVD49 = 0x49,
    /// Reserved
    _RSVD4A = 0x4A,
    /// UV Average Enable
    REG4B = 0x4B,
    /// Denoise Strength
    DNSTH = 0x4C,
    /// Reserved
    _RSVD4D = 0x4D,
    /// Reserved
    _RSVD4E = 0x4E,
    /// Matrix Coefficient 1
    MTX1 = 0x4F,
    /// Matrix Coefficient 2
    MTX2 = 0x50,
    /// Matrix Coefficient 3
    MTX3 = 0x51,
    /// Matrix Coefficient 4
    MTX4 = 0x52,
    /// Matrix Coefficient 5
    MTX5 = 0x53,
    /// Matrix Coefficient 6
    MTX6 = 0x54,
    /// Brightness Control
    BRIGHT = 0x55,
    /// Contrast Control
    CONTRAS = 0x56,
    /// Contrast Center
    CONTRAS_CENTER = 0x57,
    /// Matrix Coefficient Sign
    MTXS = 0x58,
    /// Reserved
    _RSVD59 = 0x59,
    /// Reserved
    _RSVD60 = 0x60,
    /// Reserved
    _RSVD61 = 0x61,
    /// Length Correction 1
    LCC1 = 0x62,
    /// Length Correction 2
    LCC2 = 0x63,
    /// Length Correction 3
    LCC3 = 0x64,
    /// Length Correction 4
    LCC4 = 0x65,
    /// Length Correction 5
    LCC5 = 0x66,
    /// Manual U Value
    MANU = 0x67,
    /// Manual V Value
    MANV = 0x68,
    /// Fix Gain Control
    GFIX = 0x69,
    /// G Channel AWB Gain
    GGAIN = 0x6A,
    /// PLL Control
    DLBV = 0x6B,
    /// AWB Control 3
    AWBCTR3 = 0x6C,
    /// AWB Control 2
    AWBCTR2 = 0x6D,
    /// AWB Control 1
    AWBCTR1 = 0x6E,
    /// AWB Control 0
    AWBCTR0 = 0x6F,
    /// X Scaling
    SCALING_XSC = 0x70,
    /// Y Scaling
    SCALING_YSC = 0x71,
    /// DWC Control
    SCALING_DCWCTR = 0x72,
    /// DSP Clock Divider
    SCALING_PCLK_DIV = 0x73,
    /// Digital Gain Manual Control
    REG74 = 0x74,
    /// Edge Enhancement Lower Limit
    REG75 = 0x75,
    /// Pixel Correction Enable
    REG76 = 0x76,
    /// Denoise Offset
    REG77 = 0x77,
    /// Reserved
    _RSVD78 = 0x78,
    /// Reserved
    _RSVD79 = 0x79,
    /// Gamma Curve Highest Segment Slope
    SLOP = 0x7A,
    /// Gamma Curve 1st Segment Input
    GAM1 = 0x7B,
    /// Gamma Curve 2nd Segment Input
    GAM2 = 0x7C,
    /// Gamma Curve 3rd Segment Input
    GAM3 = 0x7D,
    /// Gamma Curve 4th Segment Input
    GAM4 = 0x7E,
    /// Gamma Curve 5th Segment Input
    GAM5 = 0x7F,
    /// Gamma Curve 6th Segment Input
    GAM6 = 0x80,
    /// Gamma Curve 7th Segment Input
    GAM7 = 0x81,
    /// Gamma Curve 8th Segment Input
    GAM8 = 0x82,
    /// Gamma Curve 9th Segment Input
    GAM9 = 0x83,
    /// Gamma Curve 10th Segment Input
    GAM10 = 0x84,
    /// Gamma Curve 11th Segment Input
    GAM11 = 0x85,
    /// Gamma Curve 12th Segment Input
    GAM12 = 0x86,
    /// Gamma Curve 13th Segment Input
    GAM13 = 0x87,
    /// Gamma Curve 14th Segment Input
    GAM14 = 0x88,
    /// Gamma Curve 15th Segment Input
    GAM15 = 0x89,
    /// Reserved
    _RSVD8A = 0x8A,
    /// Reserved
    _RSVD8B = 0x8B,
    /// RGB444 Enable, Format
    RGB444 = 0x8C,
    /// Reserved
    _RSVD8D = 0x8D,
    /// Reserved
    _RSVD8E = 0x8E,
    /// Reserved
    _RSVD8F = 0x8F,
    /// Reserved
    _RSVD90 = 0x90,
    /// Reserved
    _RSVD91 = 0x91,
    /// Dummy Line low 8 bits
    DM_LNL = 0x92,
    /// Dummy Line high 8 bits
    DM_LNH = 0x93,
    /// Lens Correction Option 6
    LCC6 = 0x94,
    /// Lens Correction Option 7
    LCC7 = 0x95,
    /// Reserved
    _RSVD96 = 0x96,
    /// Reserved
    _RSVD97 = 0x97,
    /// Reserved
    _RSVD98 = 0x98,
    /// Reserved
    _RSVD99 = 0x99,
    /// Reserved
    _RSVD9A = 0x9A,
    /// Reserved
    _RSVD9B = 0x9B,
    /// Reserved
    _RSVD9C = 0x9C,
    /// 50 Hz Banding Filter Mode
    BD50ST = 0x9D,
    /// 60 Hz Banding Filter Mode
    BD60ST = 0x9E,
    /// Histogram-based AGC Control 1
    HAECC1 = 0x9F,
    /// Histogram-based AGC Control 2
    HAECC2 = 0xA0,
    /// Reserved
    _RSVDA1 = 0xA1,
    /// Pixel Clock Delay
    SCALING_PCLK_DELAY = 0xA2,
    /// Reserved
    _RSVDA3 = 0xA3,
    /// Auto Frame Rate Adjustment Control
    NT_CTRL = 0xA4,
    /// 50 Hz Banding Step Limit
    BD50MAX = 0xA5,
    /// Histogram-based AGC Control 3
    HAECC3 = 0xA6,
    /// Histogram-based AGC Control 4
    HAECC4 = 0xA7,
    /// Histogram-based AGC Control 5
    HAECC5 = 0xA8,
    /// Histogram-based AGC Control 6
    HAECC6 = 0xA9,
    /// Histogram-based AGC Control 7
    HAECC7 = 0xAA,
    /// 60 Hz Banding Step Limit
    BD60MAX = 0xAB,
    /// Strobe Control Register AC
    STR_OPT = 0xAC,
    /// R Gain for LED Output Frame
    STR_R = 0xAD,
    /// G Gain for LED Output Frame
    STR_G = 0xAE,
    /// B Gain for LED Output Frame
    STR_B = 0xAF,
    /// Reserved
    _RSVDB0 = 0xB0,
    /// ABLC Enable
    ABLC1 = 0xB1,
    /// Reserved
    _RSVDB2 = 0xB2,
    /// ALBC Target
    THL_ST = 0xB3,
    /// Reserved
    _RSVDB4 = 0xB4,
    /// ABLC Stable Range
    THL_DLT = 0xB5,
    /// Reserved
    _RSVDB6 = 0xB6,
    /// Reserved
    _RSVDB7 = 0xB7,
    /// Reserved
    _RSVDB8 = 0xB8,
    /// Reserved
    _RSVDB9 = 0xB9,
    /// Reserved
    _RSVDBA = 0xBA,
    /// Reserved
    _RSVDBB = 0xBB,
    /// Reserved
    _RSVDBC = 0xBC,
    /// Reserved
    _RSVDBD = 0xBD,
    /// Blue Channel Black Level Compensation
    AD_CHB = 0xBE,
    /// Red Channel Black Level Compensation
    AD_CHR = 0xBF,
    /// Gb Channel Black Level Compensation
    AD_CHGb = 0xC0,
    /// Gr Channel Black Level Compensation
    AD_CHGr = 0xC1,
    /// Reserved
    _RSVDC2 = 0xC2,
    /// Reserved
    _RSVDC3 = 0xC3,
    /// Reserved
    _RSVDC4 = 0xC4,
    /// Reserved
    _RSVDC5 = 0xC5,
    /// Reserved
    _RSVDC6 = 0xC6,
    /// Reserved
    _RSVDC7 = 0xC7,
    /// Reserved
    _RSVDC8 = 0xC8,
    /// Saturation Control
    SATCTR = 0xC9,
}

impl Reg {
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub const fn initial(self) -> u8 {
        #[allow(clippy::match_same_arms, reason = "values are logically independent")]
        match self {
            Self::GAIN => 0x00,
            Self::BLUE => 0x80,
            Self::RED => 0x80,
            Self::VREF => 0x03,
            Self::COM1 => 0x00,
            Self::BAVE => 0x00,
            Self::GbAVE => 0x00,
            Self::AECHH => 0x07,
            Self::RAVE => 0x00,
            Self::COM2 => 0x01,
            Self::PID => 0x76,
            Self::VER => 0x73,
            Self::COM3 => 0x00,
            Self::COM4 => 0x40,
            Self::COM5 => 0x01,
            Self::COM6 => 0x43,
            Self::AECH => 0x40,
            Self::CLKRC => 0x80,
            Self::COM7 => 0x00,
            Self::COM8 => 0x8f,
            Self::COM9 => 0x4A,
            Self::COM10 => 0x00,
            Self::_RSVD16 => 0x00,
            Self::HSTART => 0x11,
            Self::HSTOP => 0x61,
            Self::VSTRT => 0x03,
            Self::VSTOP => 0x7B,
            Self::PSHFT => 0x00,
            Self::MIDH => 0x7f,
            Self::MIDL => 0xA2,
            Self::MVFP => 0x01,
            Self::LAEC => 0x00,
            Self::ADCCTR0 => 0x04,
            Self::ADCCTR1 => 0x02,
            Self::ADCCTR2 => 0x01,
            Self::ADCCTR3 => 0x00,
            Self::AEW => 0x75,
            Self::AEB => 0x63,
            Self::VPT => 0xD4,
            Self::BBIAS => 0x80,
            Self::GbBIAS => 0x80,
            Self::_RSVD29 => 0x00,
            Self::EXHCH => 0x00,
            Self::EXHCL => 0x2B,
            Self::RBIAS => 0x80,
            Self::ADVFL => 0x00,
            Self::ADVFH => 0x00,
            Self::YAVE => 0x00,
            Self::HSYST => 0x08,
            Self::HSYEN => 0x30,
            Self::HREF => 0x80,
            Self::CHLF => 0x08,
            Self::ARBLM => 0x11,
            Self::_RSVD35 => 0x00,
            Self::_RSVD36 => 0x00,
            Self::ADC => 0x3f,
            Self::ACOM => 0x01,
            Self::OFON => 0x00,
            Self::TSLB => 0x0D,
            Self::COM11 => 0x00,
            Self::COM12 => 0x68,
            Self::COM13 => 0x88,
            Self::COM14 => 0x00,
            Self::EDGE => 0x00,
            Self::COM15 => 0xC0,
            Self::COM16 => 0x08,
            Self::COM17 => 0x00,
            Self::AWBC1 => 0x14,
            Self::AWBC2 => 0xF0,
            Self::AWBC3 => 0x45,
            Self::AWBC4 => 0x61,
            Self::AWBC5 => 0x51,
            Self::AWBC6 => 0x79,
            Self::_RSVD49 => 0x00,
            Self::_RSVD4A => 0x00,
            Self::REG4B => 0x00,
            Self::DNSTH => 0x00,
            Self::_RSVD4D => 0x00,
            Self::_RSVD4E => 0x00,
            Self::MTX1 => 0x40,
            Self::MTX2 => 0x34,
            Self::MTX3 => 0xC0,
            Self::MTX4 => 0x17,
            Self::MTX5 => 0x29,
            Self::MTX6 => 0x40,
            Self::BRIGHT => 0x00,
            Self::CONTRAS => 0x40,
            Self::CONTRAS_CENTER => 0x80,
            Self::MTXS => 0x1E,
            Self::_RSVD59 => 0x00,
            Self::_RSVD60 => 0x00,
            Self::_RSVD61 => 0x00,
            Self::LCC1 => 0x00,
            Self::LCC2 => 0x00,
            Self::LCC3 => 0x50,
            Self::LCC4 => 0x30,
            Self::LCC5 => 0x00,
            Self::MANU => 0x80,
            Self::MANV => 0x80,
            Self::GFIX => 0x00,
            Self::GGAIN => 0x00,
            Self::DLBV => 0x0A,
            Self::AWBCTR3 => 0x02,
            Self::AWBCTR2 => 0x55,
            Self::AWBCTR1 => 0xC0,
            Self::AWBCTR0 => 0x9A,
            Self::SCALING_XSC => 0x3A,
            Self::SCALING_YSC => 0x35,
            Self::SCALING_DCWCTR => 0x11,
            Self::SCALING_PCLK_DIV => 0x00,
            Self::REG74 => 0x00,
            Self::REG75 => 0x0F,
            Self::REG76 => 0x01,
            Self::REG77 => 0x10,
            Self::_RSVD78 => 0x00,
            Self::_RSVD79 => 0x00,
            Self::SLOP => 0x24,
            Self::GAM1 => 0x04,
            Self::GAM2 => 0x07,
            Self::GAM3 => 0x10,
            Self::GAM4 => 0x28,
            Self::GAM5 => 0x36,
            Self::GAM6 => 0x44,
            Self::GAM7 => 0x52,
            Self::GAM8 => 0x60,
            Self::GAM9 => 0x6C,
            Self::GAM10 => 0x78,
            Self::GAM11 => 0x8C,
            Self::GAM12 => 0x9E,
            Self::GAM13 => 0xBB,
            Self::GAM14 => 0xD2,
            Self::GAM15 => 0xE5,
            Self::_RSVD8A => 0x00,
            Self::_RSVD8B => 0x00,
            Self::RGB444 => 0x00,
            Self::_RSVD8D => 0x00,
            Self::_RSVD8E => 0x00,
            Self::_RSVD8F => 0x00,
            Self::_RSVD90 => 0x00,
            Self::_RSVD91 => 0x00,
            Self::DM_LNL => 0x00,
            Self::DM_LNH => 0x00,
            Self::LCC6 => 0x50,
            Self::LCC7 => 0x50,
            Self::_RSVD96 => 0x00,
            Self::_RSVD97 => 0x00,
            Self::_RSVD98 => 0x00,
            Self::_RSVD99 => 0x00,
            Self::_RSVD9A => 0x00,
            Self::_RSVD9B => 0x00,
            Self::_RSVD9C => 0x00,
            Self::BD50ST => 0x99,
            Self::BD60ST => 0x7F,
            Self::HAECC1 => 0xC0,
            Self::HAECC2 => 0x90,
            Self::_RSVDA1 => 0x00,
            Self::SCALING_PCLK_DELAY => 0x02,
            Self::_RSVDA3 => 0x00,
            Self::NT_CTRL => 0x00,
            Self::BD50MAX => 0x0F,
            Self::HAECC3 => 0xF0,
            Self::HAECC4 => 0xC1,
            Self::HAECC5 => 0xF0,
            Self::HAECC6 => 0xC1,
            Self::HAECC7 => 0x14,
            Self::BD60MAX => 0x0F,
            Self::STR_OPT => 0x00,
            Self::STR_R => 0x80,
            Self::STR_G => 0x80,
            Self::STR_B => 0x80,
            Self::_RSVDB0 => 0x00,
            Self::ABLC1 => 0x00,
            Self::_RSVDB2 => 0x00,
            Self::THL_ST => 0x80,
            Self::_RSVDB4 => 0x00,
            Self::THL_DLT => 0x04,
            Self::_RSVDB6 => 0x00,
            Self::_RSVDB7 => 0x00,
            Self::_RSVDB8 => 0x00,
            Self::_RSVDB9 => 0x00,
            Self::_RSVDBA => 0x00,
            Self::_RSVDBB => 0x00,
            Self::_RSVDBC => 0x00,
            Self::_RSVDBD => 0x00,
            Self::AD_CHB => 0x00,
            Self::AD_CHR => 0x00,
            Self::AD_CHGb => 0x00,
            Self::AD_CHGr => 0x00,
            Self::_RSVDC2 => 0x00,
            Self::_RSVDC3 => 0x00,
            Self::_RSVDC4 => 0x00,
            Self::_RSVDC5 => 0x00,
            Self::_RSVDC6 => 0x00,
            Self::_RSVDC7 => 0x00,
            Self::_RSVDC8 => 0x00,
            Self::SATCTR => 0xC0,
        }
    }
}
