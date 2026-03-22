#![allow(clippy::struct_field_names, reason = "Names are clearer for the macro")]

use assign_resources::assign_resources;
use embassy_stm32::{Peri, bind_interrupts, dma, exti, i2c, interrupt, peripherals, usb};

assign_resources! {
    system: System {
        wdg: IWDG,
        rcc: RCC,
        flash: FLASH,
    },
    state_machine: StateMachine {
        vbat_ok: PA9,
        vbat_exti: EXTI9,
        usb_pwr_on: PA10,
    }
    leds: Leds {
        debug_led: PA5
    },
    usb: Usb {
        usb: USB,
        dm: PA11,
        dp: PA12
    },
    flash: Flash {
        crc: CRC,
        spi: SPI3,
        dma_rx: DMA2_CH1,
        dma_tx: DMA2_CH2,

        reset_n: PB5,
        cs_n: PB4,

        sck: PC10,
        miso: PC11,
        mosi: PC12,
    },
    fpga_power: FpgaPower {
        en: PA7,
    },
    fpga: Fpga {
        spi: SPI2,
        dma_rx: DMA1_CH1,
        dma_tx: DMA1_CH2,

        gpio1: PB0,
        pwrdn_n: PB1,
        drdy: PB2,
        drdy_exti: EXTI2,

        cdone: PB10,
        cdone_exti: EXTI10,

        creset_n: PB11,
        cs_n: PB12,
        sck: PB13,
        miso: PB14,
        mosi: PB15,
    },
    camera_power: CameraPower {
        en: PA6,
    }
    camera: Camera {
        i2c: I2C3,
        mco: MCO2,
        dma_tx: DMA1_CH4,
        dma_rx: DMA1_CH5,

        scl: PC0,
        sda: PC1,
        xclk: PC2,
        reset_n: PC3,
        pwrdn: PC13,
    }
    sensors: Sensors{
        adc: ADC1,
        dma: DMA1_CH3,

        isense: PA0,
        fpga_isense: PA1,
        vsense: PA4,
    },
    comms: Comms {
        mco: MCO,
        uart: USART2,

        tx: PA2,
        rx: PA3,
        carrier: PA8,
    },
    spare: Spare {
        i2c: I2C1,

        scl: PB6,
        sda: PB7,
        gpio1: PB8,
    }
    _unused: Unused {
        pa13: PA13, // SWDIO
        pa14: PA14, // SWCLK
        pa15: PA15, // JTDI

        pb3: PB3, // SWO
        pb9: PB9, // NC

        pc4: PC4, // NC
        pc6: PC6, // NC
        pc7: PC7, // NC
        pc8: PC8, // NC
        pc9: PC9, // NC
        pc14: PC14, // NC
        pc15: PC15, // NC

        pd2: PD2, // NC

        pf0: PF0, // NC
        pf1: PF1, // NC
        pf2: PF2, // NRST
        pf3: PF3, // BOOT0
    }
}

bind_interrupts!(
    pub struct Irqs{
        EXTI2_3 => exti::InterruptHandler<interrupt::typelevel::EXTI2_3>;
        EXTI4_15 => exti::InterruptHandler<interrupt::typelevel::EXTI4_15>;

        USB_DRD_FS => usb::InterruptHandler<peripherals::USB>;

        I2C2_3_4 => i2c::EventInterruptHandler<peripherals::I2C3>,
                    i2c::ErrorInterruptHandler<peripherals::I2C3>;

        DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;

        DMA1_CHANNEL2_3 => dma::InterruptHandler<peripherals::DMA1_CH2>,
                           dma::InterruptHandler<peripherals::DMA1_CH3>;

        DMA1_CH4_7_DMA2_CH1_5_DMAMUX_OVR => dma::InterruptHandler<peripherals::DMA1_CH4>,
                                            dma::InterruptHandler<peripherals::DMA1_CH5>,
                                            dma::InterruptHandler<peripherals::DMA2_CH1>,
                                            dma::InterruptHandler<peripherals::DMA2_CH2>;
    }
);
