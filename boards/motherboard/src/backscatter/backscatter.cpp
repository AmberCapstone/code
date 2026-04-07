#include "backscatter.hpp"

#include "periph/analog_input.hpp"
#include "periph/analog_output.hpp"
#include "periph/digital.hpp"
#include "cobs.hpp"

// Proto
#include "base_station.pb.h"
#include "pb_decode.h"

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

static constexpr uint32_t RX_BUF_SIZE = 1024;
static uint8_t rxBuffer[RX_BUF_SIZE];
static uint16_t rxBufStart = 0;
static volatile uint16_t rxBufEnd = 0;

static amber::cobs::Decoder<RX_BUF_SIZE> rxDecoder;

static float logvThreshold = 0.9f;

}  // namespace

auto Init() noexcept -> void {
    logv.SetVoltage(logvThreshold);

    HAL_UART_Receive_IT(&huart3, (uint8_t*)(uartBuffer), 4);
}

auto Update1000hz() noexcept -> void {
    debug2.Set(comparator.Read());
}

auto Receive() noexcept -> void {
    bool hasData = false;

    while (rxBufStart != rxBufEnd && !hasData) {
        hasData = rxDecoder.Decode(&rxBuffer[rxBufStart], 1);
        rxBufStart = (rxBufStart + 1) % RX_BUF_SIZE;
    }

    if (hasData) {
        pb_istream_s istream =
            pb_istream_from_buffer(rxDecoder.buffer, rxDecoder.length);
        base_station_command_t cmd;
        if (pb_decode(&istream, &base_station_command_t_msg, &cmd)) {
            HandleCommand(&cmd);
        }
        rxDecoder.Reset();
    }
}

auto HandleCommand(base_station_command_t* cmd) noexcept -> void {
    return;
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

void SerialReceiveBytes(uint8_t* bytes, uint32_t len) {
    for (uint32_t i = 0; i < len; i++) {
        rxBuffer[rxBufEnd] = bytes[i];
        rxBufEnd = (rxBufEnd + 1) % RX_BUF_SIZE;
    }
}

}  // namespace backscatter

extern "C" void HAL_UART_RxCpltCallback(UART_HandleTypeDef* huart) {
    if (huart != &huart3) {
        return;
    }
    HAL_UART_Receive_IT(&huart3, (uint8_t*)(uartBuffer), 4);
    uartReceiveCount++;
}
