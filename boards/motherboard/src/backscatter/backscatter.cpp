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
}  // namespace

auto Init() noexcept -> void {
    logv.SetVoltage(0.8f);

    HAL_UART_Receive_IT(&huart3, &uartByte, 1);
}

auto Update1000hz() noexcept -> void {
    static uint8_t counter = 0;

    if (comparator.Read()) {
        counter = 10;
    }

    if (counter > 0) {
        debug2.SetHigh();
        counter--;
    } else {
        debug2.SetLow();
    }
}

auto GetUartByte() noexcept -> uint8_t {
    return uartByte;
}

auto GetReceiveCount() noexcept -> uint8_t {
    return uartReceiveCount;
}

}  // namespace backscatter

extern "C" void HAL_UART_RxCpltCallback(UART_HandleTypeDef* huart) {
    if (huart != &huart3) {
        return;
    }
    HAL_UART_Receive_IT(&huart3, &uartByte, 1);
    uartReceiveCount++;
}
