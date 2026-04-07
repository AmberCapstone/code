#include "backscatter.hpp"

#include "periph/analog_input.hpp"
#include "periph/analog_output.hpp"
#include "periph/digital.hpp"
#include "cobs.hpp"

// Proto
#include "pb_decode.h"

// CubeMX
#include "dac.h"
#include "gpio.h"
#include "usart.h"

static constexpr uint32_t RX_BUF_SIZE = 1024;
static uint8_t rxBuffer[RX_BUF_SIZE];
static uint16_t rxBufStart = 0;
static volatile uint16_t rxBufEnd = 0;

static uint8_t dummyByte;

static amber::cobs::Decoder<RX_BUF_SIZE> rxDecoder;
static backscatter_status_t lastStatus = BACKSCATTER_STATUS_INIT_ZERO;
namespace backscatter {

static amber::periph::AnalogOutput logv(hdac1, DAC1_CHANNEL_2);
static amber::periph::DigitalOutput debug2(*DEBUG2_GPIO_Port, DEBUG2_Pin);
static amber::periph::DigitalInput comparator(*BACKSCATTER_READ_GPIO_Port,
                                              BACKSCATTER_READ_Pin);

static uint16_t backscatter_bad_msgs = 0;

static float logvThreshold = 0.9f;

auto Init() noexcept -> void {
    logv.SetVoltage(logvThreshold);

    HAL_UART_Receive_IT(&huart3, &dummyByte, 1);
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
        backscatter_status_t status;
        if (pb_decode(&istream, &backscatter_status_t_msg, &status)) {
            lastStatus = status;
        } else {
            backscatter_bad_msgs++;
        }
        rxDecoder.Reset();
    }
}

auto GetReceiveCount() noexcept -> uint8_t {
    return uartReceiveCount;
}

auto GetDacThreshold() noexcept -> float {
    return logvThreshold;
}

auto GetStatus(backscatter_status_t& status) noexcept -> void {
    status = lastStatus;
}

auto GetBackscatterBadMessages() noexcept -> uint16_t {
    return backscatter_bad_msgs;
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
    rxBuffer[rxBufEnd] = dummyByte;
    rxBufEnd = (rxBufEnd + 1) % RX_BUF_SIZE;

    HAL_UART_Receive_IT(&huart3, &dummyByte, 1);
    uartReceiveCount++;
}
