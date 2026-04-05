#include "carrier.hpp"

#include "Src/power/power.hpp"
#include "lib/adf5355/adf5355.hpp"
#include "lib/periph/spi.hpp"

// CubeMX
#include "adc.h"
#include "spi.h"

namespace {

static amber::periph::DigitalOutput lpaEn(*LPA_EN_GPIO_Port, LPA_EN_Pin);
static amber::periph::DigitalOutput vcoCe(*VCO_CE_GPIO_Port, VCO_CE_Pin);
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
    if (power::GetPowerMuxState() == power::PowerMuxState::USB_POWER) {
        lpaEn.SetLow();
        vcoCe.SetLow();
    }

    lpaEn.SetHigh();
    vcoCe.SetHigh();

    HAL_Delay(10);

    ADF5355().setup();
};

auto Update_100hz() noexcept -> void {
    vcoLocked = vcoMuxOut.Read();

    if (power::GetPowerMuxState() == power::PowerMuxState::USB_POWER) {
        lpaEn.SetLow();
        vcoCe.SetLow();
        return;
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
