#include "backscatter.hpp"

#include "periph/analog_input.hpp"
#include "periph/analog_output.hpp"
#include "periph/digital.hpp"

// CubeMX
#include "dac.h"
#include "gpio.h"
#include "usart.h"

namespace backscatter {

namespace {

static amber::periph::AnalogOutput logv(hdac1, DAC1_CHANNEL_2);
static amber::periph::DigitalOutput debug2(*DEBUG2_GPIO_Port, DEBUG2_Pin);
static amber::periph::DigitalInput comparator(*BACKSCATTER_READ_GPIO_Port,
                                              BACKSCATTER_READ_Pin);
static uint16_t xCoord = 0;
static uint16_t yCoord = 0;
}  // namespace

auto Init() noexcept -> void {
    logv.SetVoltage(0.9f);

    HAL_UART_Receive_IT(&huart3, (uint8_t*)(uartBuffer), 4);
}

auto Update1000hz() noexcept -> void {
    debug2.Set(comparator.Read());
}

auto GetXCoord() noexcept -> uint16_t {
    return (uartBuffer[1] << 8) | uartBuffer[0];
}

auto GetYCoord() noexcept -> uint16_t {
    return (uartBuffer[3] << 8) | uartBuffer[2];
}

auto GetReceiveCount() noexcept -> uint8_t {
    return uartReceiveCount;
}

}  // namespace backscatter

extern "C" void HAL_UART_RxCpltCallback(UART_HandleTypeDef* huart) {
    if (huart != &huart3) {
        return;
    }
    HAL_UART_Receive_IT(&huart3, (uint8_t*)(uartBuffer), 4);
    uartReceiveCount++;
}
