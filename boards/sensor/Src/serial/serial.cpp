#include "serial.hpp"

#include "cobs.hpp"
#include "common_macros.hpp"
#include "flash/flash.hpp"
#include "sensors/sensors.hpp"
#include "state_machine/state_machine.hpp"

// Proto
#include "flash.pb.h"
#include "pb_decode.h"
#include "pb_encode.h"
#include "sensor.pb.h"

// USB
#include "usbd_cdc_if.h"
#include "usbd_core.h"

// CubeMX
#include "main.h"
#include "tim.h"
#include "usb.h"

namespace serial {

extern "C" {
extern USBD_HandleTypeDef hUsbDeviceFS;
}

// RX State
static uint32_t rx_counter = 0;

constexpr uint32_t RX_BUF_SIZE = 1024;
static uint8_t rx_buffer[RX_BUF_SIZE];
static uint16_t rx_buf_start = 0;
static volatile uint16_t rx_buf_end = 0;

static amber::cobs::Decoder<1024> rx_decoder;

// TX State
static uint8_t tx_counter = 0;
static uint8_t pb_buffer[SENSOR_STATUS_SIZE];
static uint8_t cobs_buffer[amber::cobs::MaxEncodedLength(SENSOR_STATUS_SIZE)];

static sensor_command_t last_command;
static uint16_t last_micros = 0;

static void HandleCommand(sensor_command_t* cmd);
static void SendStatus(void);

void Init(void) {
    // Wait for USB initialization to complete
    while (((volatile USBD_HandleTypeDef*)hUsbDeviceFS.pClassData) == NULL);

    HAL_TIM_Base_Start(&htim7);  // micros timer for benchmarking
}

static bool GoFast(void) {
    // Respond faster while flashing to speed up acknowledgements
    return state_machine::GetState() == SENSOR_STATE_FLASHING ||
           state_machine::GetState() == SENSOR_STATE_READOUT;
}

void Update_10hz(void) {
    if (!GoFast()) {
        SendStatus();
    }
}

void Update_100hz(void) {
    if (GoFast()) {
        SendStatus();
    }
}

void Receive(void) {
    bool has_data = false;

    uint16_t micros = __HAL_TIM_GET_COUNTER(&htim7);
    while (rx_buf_start != rx_buf_end && !has_data) {
        has_data = rx_decoder.Decode(&rx_buffer[rx_buf_start], 1);
        rx_buf_start = (rx_buf_start + 1) % RX_BUF_SIZE;
    }

    if (has_data) {
        pb_istream_s istream =
            pb_istream_from_buffer(rx_decoder.buffer, rx_decoder.length);
        sensor_command_t cmd;
        if (pb_decode(&istream, &sensor_command_t_msg, &cmd)) {
            HandleCommand(&cmd);
        }
        rx_decoder.Reset();
    }
    last_micros = __HAL_TIM_GET_COUNTER(&htim7) - micros;
}

void SendStatus(void) {
    sensor_status_t status{
        .has_tx_counter = true,
        .tx_counter = tx_counter++,
        .has_rx_counter = true,
        .rx_counter = rx_counter,
        .has_micros = true,
        .micros = last_micros,
    };

    sensors::PopulateStatus(&status);
    state_machine::PopulateStatus(&status);

    status.has_flash_status = true;
    flash::PopulateStatus(&status.flash_status);

    pb_ostream_s ostream =
        pb_ostream_from_buffer(pb_buffer, COUNTOF(pb_buffer));

    if (pb_encode(&ostream, &sensor_status_t_msg, &status)) {
        int len =
            amber::cobs::Encode(pb_buffer, ostream.bytes_written, cobs_buffer);
        CDC_Transmit(cobs_buffer, len);
    }
}

void HandleCommand(sensor_command_t* cmd) {
    rx_counter++;
    if (cmd->has_action) {
        state_machine::HandleAction(cmd->action);
    }

    if (cmd->has_page) {
        flash::ReceivePage(&cmd->page);  // 33 us
    }

    if (cmd->has_host_page_request) {
        flash::UpdateReadoutReqNumber(cmd->host_page_request);
    }

    last_command = *cmd;
}

// Modifiers
void SerialReceiveBytes(uint8_t* bytes, uint32_t len) {
    for (uint32_t i = 0; i < len; i++) {
        rx_buffer[rx_buf_end] = bytes[i];
        rx_buf_end = (rx_buf_end + 1) % RX_BUF_SIZE;
    }
}

}  // namespace serial