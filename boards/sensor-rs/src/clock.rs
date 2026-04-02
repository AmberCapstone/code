use embassy_stm32::rcc::{
    AHBPrescaler, APBPrescaler, Config, Hsi48Config, LsConfig, Pll, PllMul, PllPDiv, PllPreDiv, PllRDiv, PllSource,
    Sysclk,
    mux::{Adcsel, Clk48sel, ClockMux},
};

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
            m.clk48sel = Clk48sel::HSI48;
            m.adcsel = Adcsel::HSI; // ADC (sensors.rs) needs a slower clock
            m
        },
    }
}
