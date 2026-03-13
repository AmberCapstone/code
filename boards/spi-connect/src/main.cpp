#include "Arduino.h"
#include "SPI.h"
#include "Wire.h"
#include "cobs.hpp"

using namespace amber;

int spi_cs = 10;
SPISettings spi_settings(20000, LSBFIRST, SPI_MODE0);

uint8_t spi_buf[512];
uint8_t tx_cobs_buf[cobs::MaxEncodedLength(512)];
cobs::Decoder<512> decoder;

static const int CAPTURE_PIN = 2;

static const uint8_t OV7670_ADDR_7BIT = 0x21; // 0x42 write address >> 1 = 0x21

static const uint8_t ov7670_qvga_yuv_regs[][2] = {
    // Internal clock pre-scalar F(internal  clock) = F(input clock)/(2)
    {0x11, 0x01},   //  CLKRC

    // YUV Mode
    {0x12, 0x00},   //  COM7

    // Downsize Enable (DCW)
    {0x0C, 0x04},   //  COM3

    // DCW and scaling PCLK, manual scaling parameter, PCLK div by 2
    {0x3E, 0x19},   //  COM14

    // Horizontal scale factor of 0x3A 
    {0x70, 0x3A | 0x80},   //  SCALING_XSC

    // Vertical scale factor of 0x35 
    {0x71, 0x35 | 0x00},   //  SCALING_YSC

    // Vrtical downsample by 2, horizontal downsample by 2
    {0x72, 0x11},   //  SCALING_DCWCTR

    // PCLK Div by 2
    {0x73, 0xF1},   //  SCALING_PCLK_DIV

    // Pixel clock delay, default value
    {0xA2, 0x02}    //  SCALING_PCLK_DELAY
};

static bool ov7670_write_reg(uint8_t reg, uint8_t val) {
    Wire.beginTransmission(OV7670_ADDR_7BIT);
    Wire.write(reg);
    Wire.write(val);
    uint8_t err = Wire.endTransmission();
    return (err == 0);
}

static void ov7670_init_qvga_yuv() {
    ov7670_write_reg(0x12, 0x80);

    delay(10);

    for (size_t i = 0; i < (sizeof(ov7670_qvga_yuv_regs) / sizeof(ov7670_qvga_yuv_regs[0])); i++) {
        uint8_t reg = ov7670_qvga_yuv_regs[i][0];
        uint8_t val = ov7670_qvga_yuv_regs[i][1];
        ov7670_write_reg(reg, val);
        delay(1);
    }
}

void setup(void) {
    pinMode(spi_cs, OUTPUT);
    digitalWrite(spi_cs, HIGH);

    pinMode(CAPTURE_PIN, OUTPUT);
    digitalWrite(CAPTURE_PIN, LOW);

    Serial.begin(500000);
    SPI.begin();

    Wire.begin();
    Wire.setClock(100000);
    ov7670_init_qvga_yuv();

    delay(2000);
    digitalWrite(CAPTURE_PIN, HIGH);
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
    SPI.beginTransaction(spi_settings);
    digitalWrite(spi_cs, LOW);
    SPI.transfer(spi_buf, decoder.length);
    digitalWrite(spi_cs, HIGH);
    SPI.endTransaction();
    // spi_buf now holds the RX data from the FPGA

    // Send response to host
    uint32_t tx_len = cobs::Encode(spi_buf, decoder.length, tx_cobs_buf);
    Serial.write(tx_cobs_buf, tx_len);
}