#pragma once

#include <SPI.h>
#include "digital.hpp"

namespace amber::cc1101 {

enum class Register : uint8_t {

    // CC1101 Register Addresses
    IOCFG0 = 0x02,
    FREQ2 = 0x0D,
    FREQ1 = 0x0E,
    FREQ0 = 0x0F,
    MDMCFG4 = 0x10,
    MDMCFG3 = 0x11,
    MDMCFG2 = 0x12,
    DEVIATN = 0x15,
    MCSM0 = 0x18,
    FOCCFG = 0x19,
    AGCCTRL2 = 0x17,
    FREND0 = 0x22,
    FSCAL3 = 0x23,
    FSCAL2 = 0x24,
    FSCAL1 = 0x25,
    FSCAL0 = 0x26,
    TEST2 = 0x2C,
    TEST1 = 0x2D,
    TEST0 = 0x2E,
    PATABLE = 0x3E,

    // Command Strobes
    SRES = 0x30,
    SNOP = 0x3D,
    SCAL = 0x33,
    SRX = 0x34,
    STX = 0x35,
    SIDLE = 0x36,
};

struct Driver {

    enum class Frequency : uint8_t { MHZ_433, MHZ_915 };
    enum class Direction : uint8_t { TX, RX };

    Driver(const SPIClass&, const pin::DigitalInput&, pin::DigitalOutput&);
    ~Driver() = default;

    auto begin(const Direction) noexcept -> void;
    auto configure(const Frequency) noexcept -> void;
    auto reset() noexcept -> void;

private:

    auto writeStrobe(const Register) noexcept -> void;
    auto writeRegister(const Register, const uint8_t) noexcept -> void;

    const SPIClass& _spi;
    const pin::DigitalInput& _miso;
    pin::DigitalOutput& _cs;
};

} // namespace amber::cc1101
