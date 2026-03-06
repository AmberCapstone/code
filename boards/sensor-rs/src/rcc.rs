use embassy_stm32::rcc::*;

pub fn configure(rcc: &mut Config) {
    rcc.hsi = true;
    rcc.pll = Some(Pll {
        source: PllSource::HSI, // 16 MHz
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL7,
        divp: None,
        divq: None,
        divr: Some(PllRDiv::DIV2), // 56 MHz
    });
    rcc.sys = Sysclk::PLL1_R;
    rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true }); // needed for USB
    rcc.mux.clk48sel = mux::Clk48sel::HSI48; // USB uses ICLK
}
