#pragma once

#include <SPI.h>
#include "digital.hpp"

namespace cc1101 {

namespace {

// CC1101 Register Addresses
constexpr uint8_t IOCFG0   = 0x02;
constexpr uint8_t FREQ2    = 0x0D;
constexpr uint8_t FREQ1    = 0x0E;
constexpr uint8_t FREQ0    = 0x0F;
constexpr uint8_t MDMCFG4  = 0x10;
constexpr uint8_t MDMCFG3  = 0x11;
constexpr uint8_t MDMCFG2  = 0x12;
constexpr uint8_t DEVIATN  = 0x15;
constexpr uint8_t MCSM0    = 0x18;
constexpr uint8_t FOCCFG   = 0x19;
constexpr uint8_t AGCCTRL2 = 0x17;
constexpr uint8_t FREND0   = 0x22;
constexpr uint8_t FSCAL3   = 0x23;
constexpr uint8_t FSCAL2   = 0x24;
constexpr uint8_t FSCAL1   = 0x25;
constexpr uint8_t FSCAL0   = 0x26;
constexpr uint8_t TEST2    = 0x2C;
constexpr uint8_t TEST1    = 0x2D;
constexpr uint8_t TEST0    = 0x2E;
constexpr uint8_t PATABLE  = 0x3E;

// Command Strobes
constexpr uint8_t SRES  = 0x30;
constexpr uint8_t SNOP  = 0x3D;
constexpr uint8_t SCAL  = 0x33;
constexpr uint8_t SRX   = 0x34;
constexpr uint8_t STX   = 0x35;
constexpr uint8_t SIDLE = 0x36;

}; // namespace reg

struct Driver {

    enum class Frequency : uint8_t { MHZ_433, MHZ_915 };
    enum class Direction : uint8_t { TX, RX };

    Driver(const SPIClass&, const pin::DigitalInput&, pin::DigitalOutput&);
    ~Driver() = default;

    auto begin(const Direction) noexcept -> void;
    auto configure(const Frequency) noexcept -> void;
    auto reset() noexcept -> void;

private:

    auto writeStrobe(const uint8_t) noexcept -> void;
    auto writeRegister(const uint8_t, const uint8_t) noexcept -> void;

    const SPIClass& _spi;
    const pin::DigitalInput& _miso;
    pin::DigitalOutput& _cs;
};

} // namespace cc1101
