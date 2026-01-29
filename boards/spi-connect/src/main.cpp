#include <util/delay.h>
#include <stdint.h>
#include <avr/interrupt.h>

#include "spi.hpp"
#include "serial.hpp"


using namespace amber;

// // opcodes
// // https://github.com/damdoy/ice40_ultraplus_examples/blob/master/spi_hw/README.md
// static constexpr uint8_t OPC_NOP = 0x00;
// static constexpr uint8_t OPC_INIT = 0x01;
// static constexpr uint8_t OPC_INV32 = 0x02;
// static constexpr uint8_t OPC_LEDS = 0x04;

static constexpr uint16_t MAX_LEN = MAX_PACKET_LEN;

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

    static uint8_t tx[MAX_LEN]; 
    static uint8_t rx[MAX_LEN];

    while (1) {
        if (!serial::PacketReady()) {
            continue;
        }

        uint16_t len = serial::ReadPacket(tx);
        if (len == 0) {
            continue; 
        }

        spi::PacketTransfer(tx, rx, len);

        while (serial::IsBusy()) { }
        serial::Transmit(rx, len);
    }
}