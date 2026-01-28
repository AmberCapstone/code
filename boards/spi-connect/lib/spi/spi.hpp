#pragma once
#include <stdint.h>

namespace amber::spi {

    enum class Mode : uint8_t {
        Mode0 = 0, // CPOL=0, CPHA=0
        Mode1 = 1, // CPOL=0, CPHA=1
        Mode2 = 2, // CPOL=1, CPHA=0
        Mode3 = 3  // CPOL=1, CPHA=1
    };

    struct Config {
        bool lsb_first;         // DORD
        bool master;            // MSTR
        Mode mode;              // CPOL/CPHA
        uint8_t clock_div;      // SPR1:0, SPI2X
    };


    void Initialize(const Config& cfg);

    uint8_t Transfer(uint8_t byte);

    void CSAssert();
    void CSDeassert();

    // simultaneous transfer and read
    void PacketTransfer8(const uint8_t tx[8], uint8_t rx[8]);

} // namespace amber::spi