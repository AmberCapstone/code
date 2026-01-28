#include <util/delay.h>
#include <stdint.h>
#include <avr/interrupt.h>

#include "spi.hpp"
#include "serial.hpp"


using namespace amber;

// opcodes
// https://github.com/damdoy/ice40_ultraplus_examples/blob/master/spi_hw/README.md
static constexpr uint8_t OPC_NOP = 0x00;
static constexpr uint8_t OPC_INIT = 0x01;
static constexpr uint8_t OPC_INV32 = 0x02;
static constexpr uint8_t OPC_LEDS = 0x04;

static void transfer(const uint8_t tx[8], uint8_t rx[8]) {
    while (serial::IsBusy()) { }
    spi::PacketTransfer8(tx, rx);
    serial::Transmit(rx, 8);
    _delay_ms(1000);
}

static void tx_inv32(const uint8_t data[4]) {
    while (serial::IsBusy()) { }
    uint8_t tx[8] = {OPC_INV32, 0, 0, 0, 0, 0, 0, 0}; 
    uint8_t rx[8];
    for (uint8_t i = 0; i < 4; i++) tx[i+1] = data[i];
    spi::PacketTransfer8(tx, rx);
    serial::Transmit(rx, 8);
    _delay_ms(1000);
}

int main(void) {
    spi::Config cfg;
    cfg.lsb_first = true;
    cfg.master = true;
    cfg.mode = spi::Mode::Mode0;
    cfg.clock_div = 128;
    spi::Initialize(cfg);

    serial::Initialize();
    sei();

    _delay_ms(50);

    uint8_t tx[8] = {OPC_INIT, 0, 0, 0, 0, 0, 0, 0x11}; // init packet to start FPGA
    uint8_t rx[8];

    transfer(tx, rx);

    

    // cycle LEDs, RGB, LSB is R
    while (1) {
        tx[0] = OPC_LEDS;
        tx[7] = 0x00;
        // Red
        tx[1] = 0x01;
        transfer(tx, rx); 

        // Green
        tx[1] = 0x02;
        transfer(tx, rx); 

        // Blue
        tx[1] = 0x04;
        transfer(tx, rx); 

        tx[1] = 0x03;
        transfer(tx, rx); 

        tx[1] = 0x05;
        transfer(tx, rx); 

        tx[1] = 0x07;
        transfer(tx, rx); 

        // Off
        tx[1] = 0x00;
        transfer(tx, rx); 

        {
            uint8_t tx_32[4] = {0x00, 0x00, 0x00, 0x00}; // expect 0xFF
            tx_inv32(tx_32);
        }
        
        {
            uint8_t tx_32[4] = {0xFF, 0xFF, 0xFF, 0xFF}; // expect 0x00
            tx_inv32(tx_32);
        }

        {
            uint8_t tx_32[4] = {0xAA, 0xAA, 0xAA, 0xAA}; // expect 0x55
            tx_inv32(tx_32);
        }

        {
            uint8_t tx_32[4] = {0x12, 0x34, 0x56, 0x78}; // expect ED CB A9 87
            tx_inv32(tx_32);
        }
    }
}