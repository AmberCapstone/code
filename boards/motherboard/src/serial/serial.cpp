// #include "serial.hpp"

// #include "cobs.hpp"

// #include "usbd_cdc_if.h"
// #include "usbd_core.h"

// #include "main.h"


// namespace serial {

// extern "C" {
// extern USBD_HandleTypeDef hUsbDeviceFS;
// }

// // RX State
// static uint32_t rx_counter = 0;
// constexpr uint32_t RX_BUF_SIZE = 1024;
// static uint8_t rx_buffer[RX_BUF_SIZE];
// static uint16_t rx_buf_start = 0;
// static volatile uint16_t rx_buf_end = 0;

// static amber::cobs::Decoder<1024> rx_decoder;

// static uint8_t tx_counter = 0;
// static uint8_t pb_buffer[SENSOR_STATUS_SIZE];
// static uint8_t cobs_buffer[amber::cobs::MaxEncodedLength(SENSOR_STATUS_SIZE)];

// }  // namespace serial