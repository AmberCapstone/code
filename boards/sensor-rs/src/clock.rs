use embassy_stm32::rcc::{
    AHBPrescaler, APBPrescaler, Config, Hsi48Config, LsConfig, Pll, PllMul, PllPreDiv, PllRDiv, PllSource, Sysclk,
    mux::{self, ClockMux},
};

pub fn get_config() -> Config {
    Config {
        msi: None,
        hsi: true,
        hse: None,
        hsi48: Some(Hsi48Config { sync_from_usb: true }), // needed for USB,
        pll: Some(Pll {
            source: PllSource::HSI, // 16 MHz
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL6,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2), // 48 MHz
        }),
        sys: Sysclk::PLL1_R,
        ahb_pre: AHBPrescaler::DIV1,
        apb1_pre: APBPrescaler::DIV1,
        ls: LsConfig::off(),
        mux: {
            let mut m = ClockMux::default();
            m.clk48sel = mux::Clk48sel::HSI48;
            m
        },
    }
}
