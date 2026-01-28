#include <util/delay.h>
#include <stdint.h>

#include "spi/spi.hpp"

using namespace amber;

// opcodes
// https://github.com/damdoy/ice40_ultraplus_examples/blob/master/spi_hw/README.md
static constexpr uint8_t OPC_NOP = 0x00;
static constexpr uint8_t OPC_INIT = 0x01;
static constexpr uint8_t OPC_INV32 = 0x02;
static constexpr uint8_t OPC_LEDS = 0x04;

// static void send_cmd(uint8_t opcode, const uint8_t params[7], uint8_t rx[8]) {
//     uint8_t tx[8];
//     tx[0] = opcode;
//     for (uint8_t i = 0; i < 7; i ++) tx[i + 1] = params[i];
//     spi::PacketTransfer8(tx, rx);
// }

int main(void) {
    spi::Config cfg;
    cfg.lsb_first = true;
    cfg.master = true;
    cfg.mode = spi::Mode::Mode0;
    cfg.clock_div = 128;
    spi::Initialize(cfg);

    _delay_ms(50);

    uint8_t tx[8] = {OPC_INIT, 0, 0, 0, 0, 0, 0, 0x11}; // init packet to start FPGA
    uint8_t rx[8];
    spi::PacketTransfer8(tx, rx);
    _delay_ms(10);
    tx[0] = OPC_LEDS;
    tx[7] = 0x00;


    // cycle LEDs, RGB, LSB is R
    while (1) {
        // Red
        tx[1] = 0x01;
        spi::PacketTransfer8(tx, rx);
        _delay_ms(300);

        // Green
        tx[1] = 0x02;
        spi::PacketTransfer8(tx, rx);
        _delay_ms(300);

        // Blue
        tx[1] = 0x04;
        spi::PacketTransfer8(tx, rx);
        _delay_ms(300);

        // Off
        tx[1] = 0x00;
        spi::PacketTransfer8(tx, rx);
        _delay_ms(300);
    }
}