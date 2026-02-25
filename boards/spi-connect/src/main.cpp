#include "Arduino.h"
#include "SPI.h"
#include "cobs.hpp"

using namespace amber;

int spi_cs = 10;
SPISettings spi_settings(20000, LSBFIRST, SPI_MODE0);

uint8_t spi_buf[512];
uint8_t tx_cobs_buf[cobs::MaxEncodedLength(512)];
cobs::Decoder<512> decoder;

void setup(void) {
    pinMode(spi_cs, OUTPUT);
    digitalWrite(spi_cs, HIGH);

    Serial.begin(500000);
    SPI.begin();
}

void loop(void) {
    // Receive COBS message from host
    decoder.Reset();
    while (true) {
        int ibyte = Serial.read();
        if (ibyte != -1) {
            uint8_t byte = static_cast<uint8_t>(ibyte);
            if (decoder.Decode(&byte, 1)) {
                break;
            }
        }
    }
    // now know that decoder has a valid cobs message

    // perform a SPI transfer using decoder.
    memcpy(spi_buf, decoder.buffer, decoder.length);
    // SPI.beginTransaction(spi_settings);
    // digitalWrite(spi_cs, LOW);
    // SPI.transfer(spi_buf, decoder.length);
    // digitalWrite(spi_cs, HIGH);
    // SPI.endTransaction();
    // spi_buf now holds the RX data from the FPGA

    // Send response to host
    uint32_t tx_len = cobs::Encode(spi_buf, decoder.length, tx_cobs_buf);
    Serial.write(tx_cobs_buf, tx_len);
}