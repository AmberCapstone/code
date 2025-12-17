#include "serial.hpp"

#include "cobs.hpp"
#include "common_macros.hpp"
#include "sensors/sensors.hpp"
#include "state_machine/state_machine.hpp"

// Proto
#include "pb_decode.h"
#include "pb_encode.h"
#include "sensor.pb.h"

// USB
#include "usbd_cdc_if.h"
#include "usbd_core.h"

// CubeMX
#include "main.h"
#include "usb.h"

namespace serial {

extern "C" {
extern USBD_HandleTypeDef hUsbDeviceFS;
}

static uint8_t rx_counter = 0;

// RX State
static volatile uint16_t rx_buf_count = 0;
static uint8_t rx_buffer[1024];
static uint8_t decoded_buffer[1024];
static amber::cobs::Decoder rx_decoder(decoded_buffer);

// TX State
static uint8_t tx_counter = 0;
static uint8_t pb_buffer[SENSOR_STATUS_SIZE];
static uint8_t cobs_buffer[amber::cobs::MaxEncodedLength(SENSOR_STATUS_SIZE)];

static sensor_command_t last_command;

static void HandleCommand(sensor_command_t* cmd);

void Init(void) {
    // Wait for USB initialization to complete
    while (((volatile USBD_HandleTypeDef*)hUsbDeviceFS.pClassData) == NULL);
}

void Receive(void) {
    if (rx_buf_count == 0) {
        // No data, nothing to do.
        return;
    }

    if (rx_decoder.Decode(rx_buffer, rx_buf_count)) {
        pb_istream_s istream =
            pb_istream_from_buffer(rx_decoder.buffer, rx_decoder.length);
        sensor_command_t cmd;
        if (pb_decode(&istream, &sensor_command_t_msg, &cmd)) {
            HandleCommand(&cmd);
        }
        rx_decoder.Reset();
    }
    rx_buf_count = 0;
}

void SendStatus(void) {
    sensor_status_t status{
        .has_tx_counter = true,
        .tx_counter = tx_counter++,
        .has_rx_counter = true,
        .rx_counter = rx_counter,
    };

    sensors::PopulateStatus(&status);
    state_machine::PopulateStatus(&status);

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

    last_command = *cmd;
}

// Modifiers
void SerialReceiveBytes(uint8_t* bytes, uint32_t len) {
    memcpy(rx_buffer + rx_buf_count, bytes, len);
    rx_buf_count += len;
}

}  // namespace serial