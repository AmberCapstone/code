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
        pll: Some(Pll {
            source: PllSource::HSI,    // 16 MHz
            prediv: PllPreDiv::DIV1,   // PRE 16 MHz
            mul: PllMul::MUL6,         // PLL 96 MHz
            divp: Some(PllPDiv::DIV8), // PLLP 12 MHz
            divq: None,                // PLLQ off
            divr: Some(PllRDiv::DIV2), // PLLR 48 MHz
        }),
        sys: Sysclk::PLL1_R,
        ahb_pre: AHBPrescaler::DIV1,
        apb1_pre: APBPrescaler::DIV1,
        ls: LsConfig::off(),
        mux: {
            let mut m = ClockMux::default();
            m.clk48sel = Clk48sel::HSI48;
            m.adcsel = Adcsel::PLL1_P; // ADC (sensors.rs) needs a slower clock
            m
        },
    }
}
