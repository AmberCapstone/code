use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};

use embassy_stm32::adc::{self, Adc, Resolution};
use embassy_stm32::adc::{AdcChannel, SampleTime};
use embassy_stm32::time::Hertz;
use embassy_time::{Duration, Ticker};

use crate::proto;
use crate::resources::Sensors;
use crate::sensors::calibration::{Converter, FactoryCalibration};

mod calibration;

const fn current_sense_mv_to_ua(shunt_mohm: u32, v_gain: u32) -> u32 {
    1_000_000 / (shunt_mohm * v_gain)
}

const CLOCK: Hertz = Hertz::mhz(16); // Assumed, should match clock.rs

const VREFINT_SAMPLE_TIME: SampleTime = SampleTime::CYCLES79_5;
const TEMP_SAMPLE_TIME: SampleTime = SampleTime::CYCLES79_5;
const VBATINT_SAMPLE_TIME: SampleTime = SampleTime::CYCLES160_5;

// Check sampling times are legal at compile time
const _: () = assert!(sampling_time_ns(CLOCK, VREFINT_SAMPLE_TIME) >= 4_000); // DS14463 Rev2 Table 25
// const _: () = assert!(sampling_time_ns(CLOCK, TEMP_SAMPLE_TIME) >= 5_000); // DS14463 Rev2 Table 73
// const _: () = assert!(sampling_time_ns(CLOCK, VBATINT_SAMPLE_TIME) >= 12_000); // DS14463 Rev2 Table 74

const RESOLUTION: Resolution = Resolution::BITS12;
const EXT_SAMPLE_TIME: SampleTime = SampleTime::CYCLES79_5;

const ISENSE_UA_PER_MV: u32 = current_sense_mv_to_ua(200, 200);
const FPGA_ISENSE_UA_PER_MV: u32 = current_sense_mv_to_ua(500, 200);
const VBATEXT_VOLTAGE_DIV: u32 = 2;
const VBATINT_VOLTAGE_DIV: u32 = 3; // RM0503 Rev4 14.11

static TEMPERATURE_DEGC: AtomicI32 = AtomicI32::new(20);
static VDD_MV: AtomicU32 = AtomicU32::new(3000);
static ISENSE_UA: AtomicU32 = AtomicU32::new(0);
static FPGA_ISENSE_UA: AtomicU32 = AtomicU32::new(0);
static VBAT_MV: AtomicU32 = AtomicU32::new(3000);

#[embassy_executor::task]
pub async fn task(mut r: Sensors) {
    let mut ticker = Ticker::every(Duration::from_millis(10));

    let mut adc = Adc::new_with_config(
        r.adc,
        adc::AdcConfig {
            resolution: Some(RESOLUTION),
            ..Default::default()
        },
    );

    // adc.enable_auto_off(); // breaking the internal channels

    let factory_calibration = FactoryCalibration::new();

    loop {
        let mut buf = [0u16; 6];
        let mut chan_vrefint = adc.enable_vrefint().degrade_adc();
        let mut chan_temp = adc.enable_temperature().degrade_adc();
        let mut chan_vdd = adc.enable_vbat().degrade_adc();
        let mut chan_isense = r.isense.reborrow().degrade_adc();
        let mut chan_fpga_isense = r.fpga_isense.reborrow().degrade_adc();
        let mut chan_vbatext = r.vsense.reborrow().degrade_adc();

        // DMA read is not working - getting garbage values
        // adc.read(
        //     r.dma.reborrow(),
        //     Irqs,
        //     [
        //         (&mut chan_vrefint, SAMPLE_TIME),
        //         (&mut chan_temp, SAMPLE_TIME),
        //         (&mut chan_vdd, SAMPLE_TIME),
        //         (&mut chan_isense, SAMPLE_TIME),
        //         (&mut chan_fpga_isense, SAMPLE_TIME),
        //         (&mut chan_vbatext, SAMPLE_TIME),
        //     ]
        //     .into_iter(),
        //     &mut buf,
        // )
        // .await;

        buf[0] = adc.blocking_read(&mut chan_vrefint, VREFINT_SAMPLE_TIME);
        buf[1] = adc.blocking_read(&mut chan_temp, TEMP_SAMPLE_TIME);
        buf[2] = adc.blocking_read(&mut chan_vdd, VBATINT_SAMPLE_TIME);
        buf[3] = adc.blocking_read(&mut chan_isense, EXT_SAMPLE_TIME);
        buf[4] = adc.blocking_read(&mut chan_fpga_isense, EXT_SAMPLE_TIME);
        buf[5] = adc.blocking_read(&mut chan_vbatext, EXT_SAMPLE_TIME);

        if buf[0] != 0 {
            let conv = Converter::new(&factory_calibration, RESOLUTION, buf[0]);

            TEMPERATURE_DEGC.store(conv.temperature_degc(buf[1]), Ordering::Relaxed);
            VDD_MV.store(conv.data_to_mv(buf[2]) * VBATINT_VOLTAGE_DIV, Ordering::Relaxed);
            ISENSE_UA.store(conv.data_to_mv(buf[3]) * ISENSE_UA_PER_MV, Ordering::Relaxed);
            FPGA_ISENSE_UA.store(conv.data_to_mv(buf[4]) * FPGA_ISENSE_UA_PER_MV, Ordering::Relaxed);
            VBAT_MV.store(conv.data_to_mv(buf[5]) * VBATEXT_VOLTAGE_DIV, Ordering::Relaxed);
        }

        ticker.next().await;
    }
}

pub fn get_vbat_mv() -> u32 {
    VBAT_MV.load(Ordering::Acquire)
}

pub fn get_status() -> proto::sensor_::Measurement {
    proto::sensor_::Measurement {
        temperature_degc: TEMPERATURE_DEGC.load(Ordering::Acquire),
        vdd_mv: VDD_MV.load(Ordering::Acquire),
        vbat_mv: VBAT_MV.load(Ordering::Acquire),
        isense_ua: ISENSE_UA.load(Ordering::Acquire),
        fpga_isense_ua: FPGA_ISENSE_UA.load(Ordering::Acquire),
    }
}

const fn sampling_time_ns(clock: Hertz, sample_time: SampleTime) -> u64 {
    // Multiplying by 1000 avoids f32 and simplifies ns calculation
    let periods_x1000 = match sample_time {
        SampleTime::CYCLES1_5 => 1_500,
        SampleTime::CYCLES3_5 => 3_500,
        SampleTime::CYCLES7_5 => 7_500,
        SampleTime::CYCLES12_5 => 12_500,
        SampleTime::CYCLES19_5 => 19_500,
        SampleTime::CYCLES39_5 => 39_500,
        SampleTime::CYCLES79_5 => 79_500,
        SampleTime::CYCLES160_5 => 160_500,
    };

    let c_mhz = (clock.0 / 1_000_000) as u64;
    periods_x1000 / c_mhz
}
