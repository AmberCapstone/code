#pragma once
#include <stdint.h>

#define MAX_PACKET_LEN 643

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
    void PacketTransfer(const uint8_t tx[MAX_PACKET_LEN], uint8_t rx[MAX_PACKET_LEN], const uint16_t len);

} // namespace amber::spi