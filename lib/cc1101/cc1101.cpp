#include "cc1101.hpp"

namespace amber::cc1101 {

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

    writeRegister(FREQ2, (freq >> 16) & 0xFF);
    writeRegister(FREQ1, (freq >> 8) & 0xFF);
    writeRegister(FREQ0, (freq & 0xFF));

    writeRegister(MDMCFG2, 0x30); // modulation mode ASK/OOK, no sync

    // Set data rate (doesn't matter for CW, but set anyway)
    writeRegister(MDMCFG4, 0xC8); // Bandwidth ~100 kHz
    writeRegister(MDMCFG3, 0x93); // Data rate ~9.6 kBaud

    // Disable deviation (CW has no modulation)
    writeRegister(DEVIATN, 0x00);

    // Calibration settings
    writeRegister(MCSM0, 0x18);
    writeRegister(FOCCFG, 0x16);
    writeRegister(AGCCTRL2, 0x43);

    // Frequency synthesizer calibration
    writeRegister(FSCAL3, 0xE9);
    writeRegister(FSCAL2, 0x2A);
    writeRegister(FSCAL1, 0x00);
    writeRegister(FSCAL0, 0x1F);

    // Test settings for CW
    writeRegister(TEST2, 0x81);
    writeRegister(TEST1, 0x35);
    writeRegister(TEST0, 0x09);

    // Set output power to maximum (approximately +10 dBm)
    // PA_TABLE values: 0xC0 = +10dBm, 0x84 = +5dBm, 0x60 = 0dBm
    writeRegister(PATABLE, 0xC0);

    // Frontend configuration
    writeRegister(FREND0, 0x11);
};

auto Driver::begin(const Direction dir) noexcept -> void {
    writeStrobe(SCAL);
    delay(10);

    if (dir == Direction::TX) {
        writeStrobe(STX);
    } else if (dir == Direction::RX) {
        writeStrobe(SRX);
    }
    delay(10);
};

} // namespace amber::cc1101
