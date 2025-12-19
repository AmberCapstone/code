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
static volatile uint16_t rx_buf_count = 0;
static uint8_t rx_buffer[1024];
static uint8_t decoded_buffer[1024];
static amber::cobs::Decoder rx_decoder(decoded_buffer);

static uint32_t last_size = 0;
extern uint32_t failed_crc = 0;
static uint32_t last_msg_size = 0;

uint32_t received_bytes = 0;
uint32_t decoded_bytes = 0;

static uint16_t last_micros = 0;
static uint16_t last_wake = 0;

// TX State
static uint8_t tx_counter = 0;
static uint8_t pb_buffer[SENSOR_STATUS_SIZE];
static uint8_t cobs_buffer[amber::cobs::MaxEncodedLength(SENSOR_STATUS_SIZE)];

static sensor_command_t last_command;

static void HandleCommand(sensor_command_t* cmd);

void Init(void) {
    // Wait for USB initialization to complete
    while (((volatile USBD_HandleTypeDef*)hUsbDeviceFS.pClassData) == NULL);

    HAL_TIM_Base_Start(&htim7);
}

void Receive(void) {
    uint16_t micros = __HAL_TIM_GET_COUNTER(&htim7);
    last_micros = __HAL_TIM_GET_COUNTER(&htim7) - last_wake;
    last_wake = micros;

    __disable_irq();
    if (rx_buf_count == 0) {
        // No data, nothing to do.
        __enable_irq();
        return;
    }

    last_size = rx_buf_count;
    if (rx_decoder.Decode(rx_buffer, rx_buf_count)) {
        last_msg_size = rx_decoder.length;
        pb_istream_s istream =
            pb_istream_from_buffer(rx_decoder.buffer, rx_decoder.length);
        sensor_command_t cmd;
        // 77 us for short msg, 200 us for 256 bytes,
        if (pb_decode(&istream, &sensor_command_t_msg, &cmd)) {
            HandleCommand(&cmd);
        }
        rx_decoder.Reset();
    }
    rx_buf_count = 0;
    __disable_irq();

}  // 320 us with data, 75 without

void SendStatus(void) {
    sensor_status_t status{
        .has_tx_counter = true,
        .tx_counter = tx_counter++,
        .has_rx_counter = true,
        .rx_counter = rx_counter,
        .has_last_msg_size = true,
        .last_msg_size = last_msg_size,
        .has_rx_buf_count = true,
        .rx_buf_count = last_size - last_msg_size,
        .has_micros = true,
        .micros = last_micros,
        .has_failed_crc = true,
        .failed_crc = failed_crc,
        .has_last_action = last_command.has_action,
        .last_action = last_command.action,
        .has_received_bytes = true,
        .received_bytes = received_bytes,
        .has_decoded_bytes = true,
        .decoded_bytes = decoded_bytes,
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
}  // 300 us

void HandleCommand(sensor_command_t* cmd) {
    rx_counter++;
    if (cmd->has_action) {
        state_machine::HandleAction(cmd->action);
    }

    if (cmd->has_page) {
        flash::ReceivePage(&cmd->page);  // 33 us

        HAL_GPIO_TogglePin(LD4_GPIO_Port, LD4_Pin);
    }

    last_command = *cmd;
}

// Modifiers
void SerialReceiveBytes(uint8_t* bytes, uint32_t len) {
    memcpy(rx_buffer + rx_buf_count, bytes, len);
    rx_buf_count += len;
    received_bytes += len;
}

}  // namespace serial