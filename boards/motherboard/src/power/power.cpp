#include "power.hpp"

#include "Src/carrier/carrier.hpp"

// CubeMX
#include "adc.h"
#include "gpio.h"

namespace {

static amber::periph::DigitalInput powerMux(*MUX_ST_GPIO_Port, MUX_ST_Pin);

static uint8_t hsd1CsIdx = 0;
static uint8_t hsd2CsIdx = 0;

static std::array<float, amber::tps274160b::kNumChannels> hsd1Currents{};
static std::array<float, amber::tps274160b::kNumChannels> hsd2Currents{};
static float scatterCurrent = 0.0f;
static float p12vCurrent = 0.0f;

static power::PowerMuxState powerMuxState = power::PowerMuxState::USB_POWER;

auto P6VHsd1Config() -> amber::tps274160b::Config& {
    static amber::periph::DigitalOutput en0(*VCO_PWR_EN_GPIO_Port,
                                            VCO_PWR_EN_Pin);
    static amber::periph::DigitalOutput en3(*VGA_PWR_EN_GPIO_Port,
                                            VGA_PWR_EN_Pin);

    static amber::periph::DigitalOutput diagSel0(*P6V_HSD_ONE_SEL_GPIO_Port,
                                                 P6V_HSD_ONE_SEL_Pin);
    static amber::periph::DigitalOutput diagSel1(*P6V_HSD_ONE_SEH_GPIO_Port,
                                                 P6V_HSD_ONE_SEH_Pin);

    static amber::periph::DigitalInput fault(*P6V_HSD_ONE_nFAULT_GPIO_Port,
                                             P6V_HSD_ONE_nFAULT_Pin);
    static amber::periph::DigitalOutput diagEn(*P6V_HSD_ONE_DIAG_EN_GPIO_Port,
                                               P6V_HSD_ONE_DIAG_EN_Pin);
    static amber::periph::AnalogInput currentSense(hadc1, ADC_CHANNEL_10);

    static amber::tps274160b::Config cfg{
        .currentSenseResistor = 1800,
        .enablePins = {en0, en0, en3, en3},
        .diagSelect = {diagSel0, diagSel1},
        .fault = fault,
        .diagEn = diagEn,
        .currentSense = currentSense,
    };

    return cfg;
}

auto P6VHsd2Config() -> amber::tps274160b::Config& {
    static amber::periph::DigitalOutput en0(*GEN_EN_GPIO_Port, GEN_EN_Pin);
    static amber::periph::DigitalOutput en1(*VGA_PWR_EN_GPIO_Port,
                                            VGA_PWR_EN_Pin);
    static amber::periph::DigitalOutput en2(*LPA_PWR_EN_GPIO_Port,
                                            LPA_PWR_EN_Pin);
    static amber::periph::DigitalOutput en3(*VCO_PWR_EN_GPIO_Port,
                                            VCO_PWR_EN_Pin);

    static amber::periph::DigitalOutput diagSel0(*P6V_HSD_TWO_SEL_GPIO_Port,
                                                 P6V_HSD_TWO_SEL_Pin);
    static amber::periph::DigitalOutput diagSel1(*P6V_HSD_TWO_SEH_GPIO_Port,
                                                 P6V_HSD_TWO_SEH_Pin);

    static amber::periph::DigitalInput fault(*P6V_HSD_TWO_nFAULT_GPIO_Port,
                                             P6V_HSD_TWO_nFAULT_Pin);
    static amber::periph::DigitalOutput diagEn(*P6V_HSD_TWO_DIAG_EN_GPIO_Port,
                                               P6V_HSD_TWO_DIAG_EN_Pin);
    static amber::periph::AnalogInput currentSense(hadc1, ADC_CHANNEL_9);

    static amber::tps274160b::Config cfg{
        .currentSenseResistor = 597,
        .enablePins = {en0, en1, en2, en3},
        .diagSelect = {diagSel0, diagSel1},
        .fault = fault,
        .diagEn = diagEn,
        .currentSense = currentSense,
    };

    return cfg;
}

auto P6VScatterConfig() -> amber::tps1h100::Config& {
    static amber::periph::DigitalOutput en(*P6V_SCATTER_PWR_EN_GPIO_Port,
                                           P6V_SCATTER_PWR_EN_Pin);
    static amber::periph::DigitalOutput diagEn(
        *P6V_SCATTER_HSD_DIAG_EN_GPIO_Port, P6V_SCATTER_HSD_DIAG_EN_Pin);
    static amber::periph::AnalogInput currentSense(hadc1, ADC_CHANNEL_4);

    static amber::tps1h100::Config cfg{
        .currentSenseResistor = 750,
        .enablePin = &en,
        .diagEn = diagEn,
        .currentSense = currentSense,
    };

    return cfg;
}

auto P12VHsdConfig() -> amber::tps1h100::Config& {
    static amber::periph::DigitalOutput diagEn(*P12V_HSD_DIAG_EN_GPIO_Port,
                                               P12V_HSD_DIAG_EN_Pin);
    static amber::periph::AnalogInput currentSense(hadc1, ADC_CHANNEL_11);

    static amber::tps1h100::Config cfg{750, nullptr, diagEn, currentSense};

    return cfg;
}

auto P6VHsd1() -> amber::tps274160b::Driver& {
    static amber::tps274160b::Driver drv(P6VHsd1Config());
    return drv;
}

auto P6VHsd2() -> amber::tps274160b::Driver& {
    static amber::tps274160b::Driver drv(P6VHsd2Config());
    return drv;
}

auto P6VScatter() -> amber::tps1h100::Driver& {
    static amber::tps1h100::Driver drv(P6VScatterConfig());
    return drv;
}

// P12V is a current-sense only HSD, not software controllable
auto P12VHsd() -> amber::tps1h100::Driver& {
    static amber::tps1h100::Driver drv(P12VHsdConfig());
    return drv;
}

}  // namespace

namespace power {

auto Init() noexcept -> void {
    // if (GetPowerMuxState() == PowerMuxState::USB_POWER) {
    //     P6VHsd1().disableAll();
    //     P6VHsd2().disableAll();
    //     P6VScatter().disable();
    //     return;
    // }

    P6VHsd1().enableAll();
    P6VHsd2().enableAll();
    P6VScatter().enable();

    P6VHsd1().diagEnable(true);
    P6VHsd2().diagEnable(true);
    P6VScatter().diagEnable(true);
    P12VHsd().diagEnable(true);
}

auto Update_100hz() noexcept -> void {
    powerMuxState =
        powerMux.Read() ? PowerMuxState::BARREL_JACK : PowerMuxState::USB_POWER;

    if (carrier::GetPowerDown()) {
        P6VHsd1().disableAll();
        P6VHsd2().disableAll();
        P6VScatter().disable();
        return;
    }

    P6VHsd1().enableAll();
    P6VHsd2().enableAll();
    P6VScatter().enable();

    P6VHsd1().selectDiagPin(hsd1CsIdx);
    P6VHsd2().selectDiagPin(hsd2CsIdx);

    hsd1Currents[hsd1CsIdx] = P6VHsd1().getCurrent();
    hsd2Currents[hsd2CsIdx] = P6VHsd2().getCurrent();
    scatterCurrent = P6VScatter().getCurrent();
    p12vCurrent = P12VHsd().getCurrent();

    hsd1CsIdx = (hsd1CsIdx + 1) % amber::tps274160b::kNumChannels;
    hsd2CsIdx = (hsd2CsIdx + 1) % amber::tps274160b::kNumChannels;
}

auto GetPowerMuxState() noexcept -> PowerMuxState {
    return powerMuxState;
}

auto GetP6VHsd1Currents() noexcept
    -> const std::array<float, amber::tps274160b::kNumChannels>& {
    return hsd1Currents;
}

auto GetP6VHsd2Currents() noexcept
    -> const std::array<float, amber::tps274160b::kNumChannels>& {
    return hsd2Currents;
}

auto GetP6VScatterCurrent() noexcept -> float {
    return scatterCurrent;
}

auto GetP12VCurrent() noexcept -> float {
    return p12vCurrent;
}

}  // namespace power
