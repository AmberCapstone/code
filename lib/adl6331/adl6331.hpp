/**
 * @file adl6331.hpp
 * @author Ivan Lange
 * @brief Driver for ADL6331 RF variable gain amplifier
 * 
 * @date 2026-03-21
 */

#pragma once

#include <array>
#include <cstdint>
#include <utility>  // for std::pair

#include "periph/spi.hpp"

namespace amber::adl6331 {

static constexpr uint8_t MAX_DSA_ATTN = 24;
static constexpr uint8_t MAX_CHIP_ADDR = 7;
static constexpr uint8_t SPI_NUM_BYTES = 3;

enum class reg : uint16_t {    
    SPI_CONFIG = 0x000,
    SCRATCHPAD = 0x00A,
    MUX_LDO_EN = 0x100,
    AMP_DSA_EN = 0x101,
    AMP1_CROSS_Z = 0x107,
    AMP2_CROSS_Z = 0x113,
    STATE_A = 0x10A,
    STATE_B = 0x10B,
    STATE_C = 0x10C,
    STATE_D = 0x10D,
    MULTI_FUNC = 0x121,
    FUSE_RB0 = 0x140,
    FUSE_RB1 = 0x141,
    FUSE_RB2 = 0x142,
    FUSE_RB3 = 0x143,
    FUSE_RB4 = 0x144,
    FUSE_RB5 = 0x145,
    ACTIVE_STATE_RDBK = 0x14A,
};

enum class AttSelect : uint8_t {
    STATE_A = 0,    // ATTSEL1=0, ATTSEL0=0
    STATE_B,        // ATTSEL1=0, ATTSEL0=1
    STATE_C,        // ATTSEL1=1, ATTSEL0=0
    STATE_D,        // ATTSEL1=1, ATTSEL0=1
};

enum class AmpMode : uint8_t {
    FIXED_GAIN = 0,
    BYPASS,
};

enum class Status : uint8_t {
    OK = 0,
    SPI_FAILURE,
    INVALID_CHIP_ADDR,
    INVALID_ARG,
    INVALID_ATTN,
    NVM_LOAD_FAILURE,
    SCRATCHPAD_MISMATCH,
};

typedef struct {
    AmpMode mode1;
    AmpMode mode2;
    uint8_t attn_db;  // 0–24 dB in 1 dB steps
} StateConfig;

typedef struct {
    uint8_t chip_addr {0}; // CA[2:0], chip address 0-7 
    bool spi_4wire_en {true};
    bool spi_3v3_readback {false};
    bool amp1_en {true};
    bool amp2_en {true};
    bool dsa_en {true};

    std::array<StateConfig, 4> states = {{
        {AmpMode::BYPASS, AmpMode::BYPASS, 24},
        {AmpMode::FIXED_GAIN, AmpMode::FIXED_GAIN, 16},
        {AmpMode::FIXED_GAIN, AmpMode::FIXED_GAIN, 8},
        {AmpMode::FIXED_GAIN, AmpMode::FIXED_GAIN, 0}
    }};
} Config;

struct Driver {

    Driver(periph::Spi&, const Config&);
    ~Driver() = default;

    auto init() noexcept -> Status;
    auto softReset() noexcept -> Status;

    auto setEnable(const bool amp1, const bool amp2, const bool dsa) noexcept -> Status;
    auto setStateConfig(const AttSelect state, const StateConfig& cfg) noexcept -> Status;
    auto setDsaAttn(const AttSelect state, const uint8_t attn) noexcept -> Status;

    auto readActiveState() noexcept -> StateConfig;
    auto readFuses() noexcept -> std::array<uint8_t, 6>;

    auto readReg(const reg r) noexcept -> std::pair<Status, uint8_t>;
    auto writeReg(const reg r, const uint8_t data) noexcept -> Status;

private:
    /* SPI frame format (3 bytes, MSB first):
     * 
     * Byte 2 (first out) [23:16]:
     * [R/W][0][CA2][CA1][CA0][0][0][A8]
     * 
     * Byte 1 (second out) [15:8]:
     * [A7][A6][A5][A4][A3][A2][A1][A0]
     * 
     * Byte 0 (third out) [7:0]:
     * [D7][D6][D5][D4][D3][D2][D1][D0]
     */
    auto buildFrame(const bool write, const reg r, const uint8_t data)
        noexcept -> std::array<uint8_t, SPI_NUM_BYTES>;

    static constexpr uint8_t SOFTRESET_VAL = 0x81;
    static constexpr uint8_t FOUR_WIRE_SPI = 0x18;

    periph::Spi& _spi;
    const Config _config;

    bool _nvmLoaded {false};
};

}  // namespace amber::adl6331
