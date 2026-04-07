#include "thermal.hpp"

#include <array>

#include "Src/power/power.hpp"
#include "lib/tmp126/tmp126.hpp"
#include "periph/pwm.hpp"
#include "spi.h"
#include "tim.h"

namespace thermal {

namespace {

static amber::periph::DigitalOutput tempCs(*TEMP_CS_N_GPIO_Port, TEMP_CS_N_Pin);
static amber::periph::DigitalOutput fan2En(*FAN2_PWR_EN_GPIO_Port,
                                           FAN2_PWR_EN_Pin);

static amber::periph::Pwm fan2Pwm(htim2, TIM_CHANNEL_4);
static amber::periph::Spi tempSpi(hspi3, tempCs);

static amber::tmp126::Driver tempSensor(tempSpi, amber::tmp126::Config{});

static float currentDuty = 0.0f;

// static bool tempSensorReady = false;

// constexpr float kAlpha = 0.1f;  // EMA smoothing factor
// constexpr uint8_t kTempRegions = 5;

// float filteredTemp = 0.0f;
// float lastReading = 0.0f;

/* TMP126 is currently not functional on the motherboard*/
// struct Point {
//     float temperature;
//     uint8_t duty;
// };

// constexpr std::array<Point, 5> kThermalCurve{{
//     {40.0f, 10},
//     {55.0f, 25},
//     {70.0f, 55},
//     {85.0f, 85},
//     {95.0f, 100},
// }};

/* TMP126 is currently not functional on the motherboard*/
// auto LookupDuty(float temp) -> uint8_t {
//     if (temp <= kThermalCurve[0].temperature) {
//         return kThermalCurve[0].duty;
//     }

//     if (temp >= kThermalCurve[kTempRegions - 1].temperature) {
//         return kThermalCurve[kTempRegions - 1].duty;
//     }

//     for (size_t i = 0; i < kTempRegions - 1; ++i) {
//         const auto& p1 = kThermalCurve[i];
//         const auto& p2 = kThermalCurve[i + 1];

//         if (temp >= p1.temperature && temp <= p2.temperature) {
//             float ratio =
//                 (temp - p1.temperature) / (p2.temperature - p1.temperature);
//             return static_cast<uint8_t>(p1.duty + ratio * (p2.duty -
//             p1.duty));
//         }
//     }

//     return 100;  // should never reach here
// };

}  // namespace

auto Init() noexcept -> void {
    fan2En.SetHigh();
    fan2Pwm.Start();

    currentDuty = 80.0f;

    fan2Pwm.SetDutyCycle(currentDuty);

    // tempSensorReady = (tempSensor.init() == amber::tmp126::Status::OK);
    // if (!tempSensorReady) {
    //     fan2Pwm.SetDutyCycle(100.0f);
    // }
};

auto Update10Hz() noexcept -> void {
    if (pwr_down_flag) {
        currentDuty = 50.0f;
        fan2Pwm.SetDutyCycle(currentDuty);
    }
    /* TMP126 is currently not functional on the motherboard*/
    // const auto [status, temperature] = tempSensor.readTemperature();
    // if (status != amber::tmp126::Status::OK) {
    //     tempSensorReady = false;
    //     fan2Pwm.SetDutyCycle(100.0f);
    //     return;
    // }

    // lastReading = temperature;

    // filteredTemp += kAlpha * (temperature - filteredTemp);
    // fan2Pwm.SetDutyCycle(LookupDuty(filteredTemp));
};

// auto GetCarrierTemp() noexcept -> float {
//     return filteredTemp;
// }

auto GetCurrentFanDuty() noexcept -> float {
    return currentDuty;
};

}  // namespace thermal
