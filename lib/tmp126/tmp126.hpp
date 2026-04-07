/**
 * @file tmp126.hpp
 * @author Ivan Lange
 * @brief Driver for TMP126 SPI temperature sensor
 *
 * @date 2026-04-04
 */

#pragma once

#include <array>
#include <cstdint>
#include <utility>  // for std::pair

#include "periph/spi.hpp"

namespace amber::tmp126 {

static constexpr float LSB_DEG_C = 0.03125f;
static constexpr uint8_t SPI_NUM_BYTES = 4U;
static constexpr uint16_t CMD_READ = (1U << 8);
static constexpr uint16_t CMD_WRITE = 0U;

enum class reg : uint8_t {
    TEMP_RESULT   = 0x00,
    SLEW_RESULT   = 0x01,
    ALERT_STATUS  = 0x02,
    CONFIGURATION = 0x03,
    ALERT_ENABLE  = 0x04,
    TLOW_LIMIT    = 0x05,
    THIGH_LIMIT   = 0x06,
    HYSTERESIS    = 0x07,
    SLEW_LIMIT    = 0x08,
    UNIQUE_ID1    = 0x09,
    UNIQUE_ID2    = 0x0A,
    UNIQUE_ID3    = 0x0B,
    DEVICE_ID     = 0x0C,
};

enum class ConvPeriod : uint8_t {
    MS_6      = 0b000,
    MS_31_25  = 0b001,
    MS_62_5   = 0b010,
    MS_125    = 0b011,
    MS_250    = 0b100,
    MS_500    = 0b101,
    S_1       = 0b110,
    S_2       = 0b111,
};

enum class Status : uint8_t {
    OK = 0,
    SPI_FAILURE,
    INVALID_PERIOD,
    DATA_NOT_READY,
};

struct Config {
    ConvPeriod conv_period {ConvPeriod::MS_62_5};
    bool averaging {true};
};

struct Driver {
    Driver(periph::Spi& spi, const Config& config);
    ~Driver() = default;

    auto init() noexcept -> Status;

    auto softReset() noexcept -> Status;
    auto setConvPeriod(ConvPeriod period) noexcept -> Status;
    auto readTemperature() noexcept -> std::pair<Status, float>;
    auto isDataReady() noexcept -> bool;

    auto readReg (reg r) noexcept -> std::pair<Status, uint16_t>;
    auto writeReg(reg r, uint16_t data) noexcept -> Status;

private:
    auto buildFrame(bool write, reg r, uint16_t data) noexcept -> std::array<uint8_t, SPI_NUM_BYTES>;

    static auto rawToDegreesC(uint16_t raw) noexcept -> float;

    static constexpr uint16_t CFG_RESET_BIT = (1U << 8);
    static constexpr uint16_t CFG_AVG_BIT = (1U << 7);
    static constexpr uint16_t CFG_MODE_BIT = (1U << 3);
    static constexpr uint16_t CFG_PERIOD_MASK = 0x0007U;

    static constexpr uint16_t ALERT_DATA_READY = (1U << 0);

    periph::Spi& _spi;
    const Config _config;
};

} // namespace amber::tmp126
