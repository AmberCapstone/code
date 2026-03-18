use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};

use embassy_stm32::adc::{self, Adc, Resolution};
use embassy_stm32::adc::{AdcChannel, SampleTime};
use embassy_time::{Duration, Ticker};

use crate::proto;
use crate::resources::{Irqs, Sensors};
use crate::sensors::calibration::{Converter, FactoryCalibration};

mod calibration;

const fn current_sense_mv_to_ua(shunt_mohm: u32, v_gain: u32) -> u32 {
    1_000_000 / (shunt_mohm * v_gain)
}

const ISENSE_UA_PER_MV: u32 = current_sense_mv_to_ua(200, 200);
const FPGA_ISENSE_UA_PER_MV: u32 = current_sense_mv_to_ua(500, 200);
const VBAT_VOLTAGE_DIV: u32 = 2;

static TEMPERATURE_DEGC: AtomicI32 = AtomicI32::new(20);
static VDD_MV: AtomicU32 = AtomicU32::new(3000);
static ISENSE_UA: AtomicU32 = AtomicU32::new(0);
static FPGA_ISENSE_UA: AtomicU32 = AtomicU32::new(0);
static VBAT_MV: AtomicU32 = AtomicU32::new(3000);

#[embassy_executor::task]
pub async fn task(mut r: Sensors) {
    const RESOLUTION: Resolution = Resolution::BITS12;
    const SAMPLE_TIME: SampleTime = SampleTime::CYCLES160_5;

    let mut ticker = Ticker::every(Duration::from_millis(10));

    let mut adc = Adc::new_with_config(
        r.adc,
        adc::AdcConfig {
            resolution: Some(RESOLUTION),
            ..Default::default()
        },
    );
    adc.enable_auto_off();

    let factory_calibration = FactoryCalibration::new();

    let mut chan_vrefint = adc.enable_vrefint().degrade_adc();
    let mut chan_temp = adc.enable_temperature().degrade_adc();
    let mut chan_vdd = adc.enable_vbat().degrade_adc();
    let mut chan_isense = r.isense.degrade_adc();
    let mut chan_fpga_isense = r.fpga_isense.degrade_adc();
    let mut chan_vbatext = r.vsense.degrade_adc();

    loop {
        let mut buf = [0u16; 6];
        adc.read(
            r.dma.reborrow(),
            Irqs,
            [
                (&mut chan_vrefint, SAMPLE_TIME),
                (&mut chan_temp, SAMPLE_TIME),
                (&mut chan_vdd, SAMPLE_TIME),
                (&mut chan_isense, SAMPLE_TIME),
                (&mut chan_fpga_isense, SAMPLE_TIME),
                (&mut chan_vbatext, SAMPLE_TIME),
            ]
            .into_iter(),
            &mut buf,
        )
        .await;

        let conv = Converter::new(&factory_calibration, RESOLUTION, buf[0]);

        TEMPERATURE_DEGC.store(conv.temperature_degc(buf[1]), Ordering::Relaxed);
        VDD_MV.store(conv.data_to_mv(buf[2]), Ordering::Relaxed);
        ISENSE_UA.store(conv.data_to_mv(buf[3]) * ISENSE_UA_PER_MV, Ordering::Relaxed);
        FPGA_ISENSE_UA.store(conv.data_to_mv(buf[4]) * FPGA_ISENSE_UA_PER_MV, Ordering::Relaxed);
        VBAT_MV.store(conv.data_to_mv(buf[5]) * VBAT_VOLTAGE_DIV, Ordering::Relaxed);

        ticker.next().await;
    }
}

pub fn get_vbat_mv() -> u32 {
    VBAT_MV.load(Ordering::Acquire)
}

pub fn get_measurements() -> proto::sensor_::Measurement {
    proto::sensor_::Measurement {
        temperature_degc: TEMPERATURE_DEGC.load(Ordering::Acquire),
        vdd_mv: VDD_MV.load(Ordering::Acquire),
        vbat_mv: VBAT_MV.load(Ordering::Acquire),
        isense_ua: ISENSE_UA.load(Ordering::Acquire),
        fpga_isense_ua: FPGA_ISENSE_UA.load(Ordering::Acquire),
    }
}
