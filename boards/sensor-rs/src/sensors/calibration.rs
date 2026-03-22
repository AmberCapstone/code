use embassy_stm32::adc::{Resolution, VREF_CALIB_MV, resolution_to_max_count};

pub mod reg {
    pub const TS_CAL1: *const u16 = 0x1FFF_6E68 as *const u16;
    pub const TS_CAL2: *const u16 = 0x1FFF_6E8A as *const u16;
    pub const VREFINT_CAL: *const u16 = 0x1FFF_6EA4 as *const u16;
}

const VREFINT_CAL_RESOLUTION: Resolution = Resolution::BITS12;

const TS_CAL1_TEMP: i32 = 30;
const TS_CAL2_TEMP: i32 = 130;
const TEMPSENSOR_CAL_VREFANALOG_MV: u32 = 3000;
const TEMPSENSOR_CAL_RESOLUTION: Resolution = Resolution::BITS12;

/// `__LL_ADC_CONVERT_DATA_RESOLUTION`
const fn convert_data_resolution(data: u16, res_current: Resolution, res_target: Resolution) -> u16 {
    // Rust (resolution.to_bits() << 1) == C ((__ADC_RESOLUTION_CURRENT__) >> (ADC_CFGR1_RES_BITOFFSET_POS - 1UL))
    let shift_up = res_current.to_bits() << 1;
    let shift_down = res_target.to_bits() << 1;
    (data << shift_up) >> shift_down
}

pub struct FactoryCalibration {
    ts_cal1_raw: i32,
    ts_cal2_raw: i32,
    vrefint_cal: u16,
}

impl FactoryCalibration {
    pub fn new() -> Self {
        Self {
            ts_cal1_raw: i32::from(unsafe { core::ptr::read_unaligned(reg::TS_CAL1) }),
            ts_cal2_raw: i32::from(unsafe { core::ptr::read_unaligned(reg::TS_CAL2) }),
            vrefint_cal: unsafe { core::ptr::read_unaligned(reg::VREFINT_CAL) },
        }
    }

    /// `__LL_ADC_CALC_VREFANALOG_VOLTAGE`
    fn calc_vrefanalog_mv(&self, vrefint_adc_data: u16, resolution: Resolution) -> u32 {
        u32::from(self.vrefint_cal) * VREF_CALIB_MV
            / u32::from(convert_data_resolution(
                vrefint_adc_data,
                resolution,
                VREFINT_CAL_RESOLUTION,
            ))
    }

    /// `__LL_ADC_CALC_TEMPERATURE`
    fn calc_temperature_degc(&self, vrefanalog_mv: u32, ts_adc_data: u16, resolution: Resolution) -> i32 {
        let ts_data = i32::from(convert_data_resolution(
            ts_adc_data,
            resolution,
            TEMPSENSOR_CAL_RESOLUTION,
        )) * vrefanalog_mv.cast_signed()
            / TEMPSENSOR_CAL_VREFANALOG_MV.cast_signed();

        (ts_data - self.ts_cal1_raw) * (TS_CAL2_TEMP - TS_CAL1_TEMP) / (self.ts_cal2_raw - self.ts_cal1_raw)
            + TS_CAL1_TEMP
    }
}

pub struct Converter<'a> {
    calibration: &'a FactoryCalibration,
    resolution: Resolution,
    vrefanalog_mv: u32,
}

impl<'a> Converter<'a> {
    pub fn new(calibration: &'a FactoryCalibration, resolution: Resolution, vrefint_adc_data: u16) -> Self {
        // Copied from __LL_ADC_CALC_VREFANALOG_VOLTAGE
        let vrefanalog_mv = calibration.calc_vrefanalog_mv(vrefint_adc_data, resolution);

        Self {
            calibration,
            resolution,
            vrefanalog_mv,
        }
    }

    /// `__LL_ADC_CALC_DATA_TO_VOLTAGE`
    pub fn data_to_mv(&self, data: u16) -> u32 {
        u32::from(data) * self.vrefanalog_mv / resolution_to_max_count(self.resolution)
    }

    pub fn temperature_degc(&self, ts_adc_data: u16) -> i32 {
        self.calibration
            .calc_temperature_degc(self.vrefanalog_mv, ts_adc_data, self.resolution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_resolution() {
        use Resolution::*;
        let cases = [
            ((0x0fff, BITS12), (0x003f, BITS6)),
            ((0x0fff, BITS12), (0x00ff, BITS8)),
            ((0x0fff, BITS12), (0x03ff, BITS10)),
            ((0x0fff, BITS12), (0x0fff, BITS12)),
            ((0x003f, BITS6), (0x003f, BITS6)),
            ((0x003f, BITS6), (0x00f7, BITS8)),
            ((0x003f, BITS6), (0x03f0, BITS10)),
            ((0x003f, BITS6), (0x0f70, BITS12)),
        ];

        for (from, to) in cases {
            assert_eq!(convert_data_resolution(from.0, from.1, to.1), to.0);

            // Conversions are lossy so we can't test the cases in reverse
        }
    }
}
