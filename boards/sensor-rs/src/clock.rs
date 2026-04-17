use embassy_stm32::rcc::{
    AHBPrescaler, APBPrescaler, Config, Hsi48Config, LsConfig, McoPrescaler, McoSource, Sysclk,
    mux::{Adcsel, Clk48sel, ClockMux},
};
use embassy_stm32::time::Hertz;

pub struct McoClocks {
    pub source: McoSource,
    pub carrier_div: McoPrescaler,
    pub camera_div: McoPrescaler,
}

pub const MCO_CLOCKS: McoClocks = McoClocks {
    source: McoSource::HSI48,
    carrier_div: McoPrescaler::DIV8,
    camera_div: McoPrescaler::DIV4,
};

pub const SYS_FREQ: Hertz = Hertz::mhz(16);

pub fn get_config() -> Config {
    Config {
        msi: None,
        hsi: true,
        hse: None,

        hsi48: Some(Hsi48Config { sync_from_usb: true }), // needed for USB,

        pll: None,
        sys: Sysclk::HSI,
        ahb_pre: AHBPrescaler::DIV1,
        apb1_pre: APBPrescaler::DIV1,
        ls: LsConfig::off(),
        mux: {
            let mut m = ClockMux::default();
            m.clk48sel = Clk48sel::HSI48; // for usb
            m.adcsel = Adcsel::SYS; // ADC (sensors.rs) needs a slower clock
            m
        },
    }
}
