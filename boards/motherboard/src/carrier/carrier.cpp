#include "carrier.hpp"

#include "Src/power/power.hpp"
#include "lib/adf5355/adf5355.hpp"
#include "lib/periph/spi.hpp"

// CubeMX
#include "adc.h"
#include "spi.h"

namespace {

static amber::periph::DigitalOutput lpaEn(*LPA_EN_GPIO_Port, LPA_EN_Pin);
static amber::periph::DigitalOutput lnaEn(*LNA_EN_GPIO_Port, LNA_EN_Pin);
static amber::periph::DigitalOutput logampEn(*LOGAMP_EN_GPIO_Port,
                                             LOGAMP_EN_Pin);
static amber::periph::DigitalOutput vcoCe(*VCO_CE_GPIO_Port, VCO_CE_Pin);
static amber::periph::DigitalOutput fan1En(*FAN1_PWR_EN_GPIO_Port,
                                           FAN1_PWR_EN_Pin);
static amber::periph::DigitalOutput fan1Pwm(*FAN1_PWN_GPIO_Port, FAN1_PWN_Pin);
static amber::periph::DigitalOutput vgaEn(*VGA_EN_GPIO_Port, VGA_EN_Pin);
static amber::periph::DigitalInput vcoMuxOut(*VCO_MUXOUT_GPIO_Port,
                                             VCO_MUXOUT_Pin);
static amber::periph::AnalogInput lpaPowerDetect(hadc1, ADC_CHANNEL_7);

static bool vcoLocked = false;
static bool powerOffRequested = false;

auto ADF5355Config() -> amber::adf5355::Driver::InitParam& {
    static amber::adf5355::Driver::InitParam cfg{};
    return cfg;
}

auto ADF5355() -> amber::adf5355::Driver& {
    static amber::periph::DigitalOutput vcoLe(*VCO_LE_GPIO_Port, VCO_LE_Pin);
    static amber::periph::Spi spi2(hspi2, vcoLe);
    static amber::adf5355::Driver drv(spi2, ADF5355Config());
    return drv;
}

}  // namespace

namespace carrier {

auto Init() noexcept -> void {
    lpaEn.SetHigh();
    lnaEn.SetHigh();
    logampEn.SetLow();
    fan1En.SetHigh();
    fan1Pwm.SetHigh();
    vcoCe.SetHigh();
    vgaEn.SetHigh();

    HAL_Delay(10);

    ADF5355().setup();
};

auto Update_100hz() noexcept -> void {
    vcoLocked = vcoMuxOut.Read();

    if (powerOffRequested) {
        lpaEn.SetLow();
        lnaEn.SetLow();
        vcoCe.SetLow();
        fan1En.SetLow();
        logampEn.SetHigh();
        vgaEn.SetLow();
        return;
    } else {
        lpaEn.SetHigh();
        lnaEn.SetHigh();
        vcoCe.SetHigh();
        fan1En.SetHigh();
        logampEn.SetLow();
        vgaEn.SetHigh();
    }

    if (pwr_down_flag) {
        vcoCe.SetLow();
        powerOffRequested = true;
        return;
    }
};

auto GetVcoLocked() noexcept -> bool {
    return vcoLocked;
};

auto GetPowerDown() noexcept -> bool {
    return powerOffRequested;
};

}  // namespace carrier
