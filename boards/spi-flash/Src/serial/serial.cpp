#include "serial.hpp"

#include "cobs.hpp"
#include "common_macros.hpp"
#include "sensors/sensors.hpp"

// Proto
#include "pb_decode.h"
#include "pb_encode.h"
#include "spi_flash.pb.h"

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
static uint8_t pb_buffer[SPI_FLASH_STATUS_SIZE];
static uint8_t
    cobs_buffer[amber::cobs::MaxEncodedLength(SPI_FLASH_STATUS_SIZE)];

static spi_flash_command_t last_command;

static void HandleCommand(spi_flash_command_t* cmd);

void Init(void) {
    // Wait for USB initialization to complete
    while (hUsbDeviceFS.pClassData == NULL) {
        // prevent compiler from optimizing out the loop condition
        asm("nop");
    }
}

void Receive(void) {
    if (rx_buf_count == 0) {
        return;
    }

    if (rx_decoder.Decode(rx_buffer, rx_buf_count)) {
        pb_istream_s istream =
            pb_istream_from_buffer(rx_decoder.buffer, rx_decoder.length);
        spi_flash_command_t cmd;
        if (pb_decode(&istream, &spi_flash_command_t_msg, &cmd)) {
            HandleCommand(&cmd);
        }
        rx_decoder.Reset();
    }
    rx_buf_count = 0;
}

void SendStatus(void) {
    spi_flash_status_t status{
        .has_tx_counter = true,
        .tx_counter = tx_counter++,
        .has_rx_counter = true,
        .rx_counter = rx_counter,
        .has_echo = last_command.has_value,
        .echo = last_command.value,
    };

    sensors::PopulateStatus(&status);

    pb_ostream_s ostream =
        pb_ostream_from_buffer(pb_buffer, COUNTOF(pb_buffer));

    if (pb_encode(&ostream, &spi_flash_status_t_msg, &status)) {
        int len =
            amber::cobs::Encode(pb_buffer, ostream.bytes_written, cobs_buffer);
        CDC_Transmit(cobs_buffer, len);
    }
}

void HandleCommand(spi_flash_command_t* cmd) {
    rx_counter++;
    last_command = *cmd;
}

// Modifiers
void SerialReceiveBytes(uint8_t* bytes, uint32_t len) {
    memcpy(rx_buffer + rx_buf_count, bytes, len);
    rx_buf_count += len;
}

}  // namespace serial