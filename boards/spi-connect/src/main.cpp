#include "Arduino.h"
#include "SPI.h"
#include "cobs.hpp"

using namespace amber;

// // opcodes
// //
// https://github.com/damdoy/ice40_ultraplus_examples/blob/master/spi_hw/README.md
// static constexpr uint8_t OPC_NOP = 0x00;
// static constexpr uint8_t OPC_INIT = 0x01;
// static constexpr uint8_t OPC_INV32 = 0x02;
// static constexpr uint8_t OPC_LEDS = 0x04;

int spi_cs = 13;
SPISettings spi_settings(100000, LSBFIRST, SPI_MODE0);

uint8_t spi_buf[512];
uint8_t tx_cobs_buf[cobs::MaxEncodedLength(512)];

int main(void) {
    pinMode(spi_cs, OUTPUT);
    digitalWrite(spi_cs, HIGH);

    Serial.begin(500000);
    SPI.begin();

    cobs::Decoder<512> decoder;

    while (1) {
        // Receive COBS message from host
        while (true) {
            uint8_t byte = Serial.read();
            if (decoder.Decode(&byte, 1)) {
                break;
            }
        }
        // now know that decoder has a valid cobs message

        // perform a SPI transfer using decoder.
        memcpy(spi_buf, decoder.buffer, decoder.length);
        digitalWrite(spi_cs, LOW);
        SPI.transfer(spi_buf, decoder.length);
        digitalWrite(spi_cs, HIGH);
        // spi_buf now holds the RX data from the FPGA

        // Send response to host
        uint32_t tx_len = cobs::Encode(spi_buf, decoder.length, tx_cobs_buf);
        Serial.write(tx_cobs_buf, tx_len);
    }
}