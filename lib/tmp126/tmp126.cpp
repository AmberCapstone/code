#include "tmp126.hpp"

namespace amber::tmp126 {

Driver::Driver(periph::Spi& spi, const Config& config)
    : _spi(spi), _config(config) {}

auto Driver::init() noexcept -> Status {
    uint16_t cfg = 0x0000U;

    if (_config.averaging) {
        cfg |= CFG_AVG_BIT;
    }

    cfg &= ~CFG_MODE_BIT;

    cfg |= (static_cast<uint16_t>(_config.conv_period) & CFG_PERIOD_MASK);

    auto status = writeReg(reg::CONFIGURATION, cfg);
    if (status != Status::OK) {
        return status;
    }

    status = writeReg(reg::THIGH_LIMIT, 0x500);

    return Status::OK;
}

auto Driver::softReset() noexcept -> Status {
    return writeReg(reg::CONFIGURATION, CFG_RESET_BIT);
}

auto Driver::setConvPeriod(const ConvPeriod period) noexcept -> Status {
    auto [status, current] = readReg(reg::CONFIGURATION);
    if (status != Status::OK) {
        return status;
    }

    uint16_t updated = (current & ~CFG_PERIOD_MASK)
                     | (static_cast<uint16_t>(period) & CFG_PERIOD_MASK);

    return writeReg(reg::CONFIGURATION, updated);
}

auto Driver::readTemperature() noexcept -> std::pair<Status, float> {
    auto [status, raw] = readReg(reg::TEMP_RESULT);
    if (status != Status::OK) {
        return {status, 0.0f};
    }
    return {Status::OK, rawToDegreesC(raw)};
}

auto Driver::isDataReady() noexcept -> bool {
    auto [status, val] = readReg(reg::ALERT_STATUS);
    if (status != Status::OK) {
        return false;
    }
    return (val & ALERT_DATA_READY) != 0U;
}

auto Driver::readReg(const reg r) noexcept -> std::pair<Status, uint16_t> {
    auto txFrame = buildFrame(false, r, 0x0000U);
    std::array<uint8_t, SPI_NUM_BYTES> rxFrame {};

    auto status = _spi.transceive(txFrame.data(), rxFrame.data(), SPI_NUM_BYTES);
    if (status != HAL_OK) {
        return {Status::SPI_FAILURE, 0U};
    }

    uint16_t data = (static_cast<uint16_t>(rxFrame[2]) << 8)
                  |  static_cast<uint16_t>(rxFrame[3]);

    return {Status::OK, data};
}

auto Driver::writeReg(const reg r, const uint16_t data) noexcept -> Status {
    auto frame = buildFrame(true, r, data);

    auto halStatus = _spi.transmit(frame.data(), SPI_NUM_BYTES);
    if (halStatus != HAL_OK) {
        return Status::SPI_FAILURE;
    }

    return Status::OK;
}

auto Driver::buildFrame(const bool write, const reg r, const uint16_t data = 0U)
    noexcept -> std::array<uint8_t, SPI_NUM_BYTES>
{
    std::array<uint8_t, SPI_NUM_BYTES> frame{0};

    uint16_t cmd = 0U;

    if (!write) {
        cmd |= (1U << 8);                    // R/W = 1 for Read
    }
    // Auto-increment bit (bit 9) can be added here if you need burst reads later:
    // if (autoIncrement) cmd |= (1U << 9);

    cmd |= (static_cast<uint16_t>(r) & 0xFFU);  // Sub-Address goes in bits [7:0]

    frame[0] = static_cast<uint8_t>(cmd >> 8);
    frame[1] = static_cast<uint8_t>(cmd & 0xFFU);

    if (!write) {
        // Read → send dummy bytes (peripheral drives SIO)
        frame[2] = 0x00U;
        frame[3] = 0x00U;
    } else {
        // Write → send actual data
        frame[2] = static_cast<uint8_t>(data >> 8);
        frame[3] = static_cast<uint8_t>(data & 0xFFU);
    }

    return frame;
}

auto Driver::rawToDegreesC(const uint16_t raw) noexcept -> float {
    auto signed14 = static_cast<int16_t>(raw) >> 2;
    return static_cast<float>(signed14) * LSB_DEG_C;
}

} // namespace amber::tmp126
