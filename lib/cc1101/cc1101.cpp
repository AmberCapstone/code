#include "cc1101.hpp"

namespace cc1101 {

Driver::Driver(
    const SPIClass& spi,
    const pin::DigitalInput& miso,
    pin::DigitalOutput& cs
) : _spi(spi), _cs(cs), _miso(miso) {};

auto Driver::writeStrobe(const uint8_t strobe) noexcept -> void {
    _cs.setLow();
    while(_miso.read());
    _spi.transfer(strobe);
    _cs.setHigh();
};

auto Driver::writeRegister(const uint8_t addr, const uint8_t value) noexcept -> void {
    _cs.setLow();
    while(_miso.read());
    _spi.transfer(addr);
    _spi.transfer(value);
    _cs.setHigh();
};

auto Driver::reset() -> void {
    _cs.setLow();
    delayMicroseconds(10);
    _cs.setHigh();
    delayMicroseconds(40);

    _cs.setLow();
    while(_miso.read());

    _spi.transfer(SRES);
    while(_miso.read());

    _cs.setHigh();
    delay(1);
}

auto Driver::configure(const Frequency carrier) noexcept -> void {
    uint32_t freq {};

    switch (carrier) {
        case Frequency::MHZ_433:
            freq = 0x10A762;
            break;
        case Frequency::MHZ_915:
            freq = 0x23313B;
            break;
        default:
            break;
    }

    writeRegister(Register::FREQ2, (freq >> 16) & 0xFF);
    writeRegister(Register::FREQ1, (freq >> 8) & 0xFF);
    writeRegister(Register::FREQ0, (freq & 0xFF));

    writeRegister(Register::MDMCFG2, 0x30); // modulation mode ASK/OOK, no sync

    // Set data rate (doesn't matter for CW, but set anyway)
    writeRegister(Register::MDMCFG4, 0xC8); // Bandwidth ~100 kHz
    writeRegister(Register::MDMCFG3, 0x93); // Data rate ~9.6 kBaud

    // Disable deviation (CW has no modulation)
    writeRegister(Register::DEVIATN, 0x00);

    // Calibration settings
    writeRegister(Register::MCSM0, 0x18);
    writeRegister(Register::FOCCFG, 0x16);
    writeRegister(Register::AGCCTRL2, 0x43);

    // Frequency synthesizer calibration
    writeRegister(Register::FSCAL3, 0xE9);
    writeRegister(Register::FSCAL2, 0x2A);
    writeRegister(Register::FSCAL1, 0x00);
    writeRegister(Register::FSCAL0, 0x1F);

    // Test settings for CW
    writeRegister(Register::TEST2, 0x81);
    writeRegister(Register::TEST1, 0x35);
    writeRegister(Register::TEST0, 0x09);

    // Set output power to maximum (approximately +10 dBm)
    // PA_TABLE values: 0xC0 = +10dBm, 0x84 = +5dBm, 0x60 = 0dBm
    writeRegister(Register::PATABLE, 0xC0);

    // Frontend configuration
    writeRegister(Register::FREND0, 0x11);
};

auto Driver::begin(const Direction dir) noexcept -> void {
    writeStrobe(SCAL);
    delay(10);

    if (dir == Direction::TX) {
        writeStrobe(Register::STX);
    } else if (dir == Direction::RX) {
        writeStrobe(Register::SRX);
    }
    delay(10);
};

} // namespace cc1101
