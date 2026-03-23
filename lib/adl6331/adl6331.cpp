#include "adl6331.hpp"

namespace amber::adl6331 {

Driver::Driver(const periph::Spi& spi, const Config& config)
    : _spi(spi), _config(config) {}

auto Driver::init() noexcept -> Status {
    // validate the chip address provided in the config
    if (_config.chip_addr > MAX_CHIP_ADDR) {
        return Status::INVALID_CHIP_ADDR;
    }

    // Step 1: Perform a software reset
    auto status = softReset();
    if (status != Status::OK) {
        return status;
    }

    // Step 2: Configure the SPI mode from the config provided
    uint8_t spiConfigVal {0};
    if (_config.spi_4wire_en) {
        spiConfigVal |= FOUR_WIRE_SPI;
    }

    status = writeReg(reg::SPI_CONFIG, spiConfigVal);
    if (status != Status::OK) {
        return status;
    }

    // Steps 3-5: Write to the scratchpad register 3 times
    for (uint8_t i = 0; i < 3U; i++) {
        auto status = writeReg(reg::SCRATCHPAD, static_cast<uint8_t>(i));
        if (status != Status::OK) {
            return status;
        }
    }

    // NVM values are now successfully loaded
    _nvmLoaded = true;

    // Step 6: Enable AMP1, AMP2, and DSA according to the config provided
    status = setEnable(_config.amp1_en, _config.amp2_en, _config.dsa_en);
    if (status != Status::OK) {
        return status;
    }

    return Status::OK;
};

auto Driver::softReset() noexcept -> Status {
    auto status = writeReg(reg::SPI_CONFIG, SOFTRESET_VAL);
    if (status != Status::OK) {
        return status;
    }
    return Status::OK;
};

auto Driver::setEnable(const bool amp1, const bool amp2, const bool dsa) noexcept -> Status {
    if (!_nvmLoaded) {
        return Status::NVM_LOAD_FAILURE;
    }

    uint8_t ampEnable = (amp2 << 2)
                      | (dsa << 1)
                      | amp1;

    auto status = writeReg(reg::AMP_DSA_EN, ampEnable);
    if (status != Status::OK) {
        return status;
    }

    return Status::OK;
};    

auto Driver::setStateConfig(const AttSelect state, const StateConfig& cfg) noexcept -> Status {

    if (cfg.attn_db > MAX_DSA_ATTN) {
        return Status::INVALID_ATTN;
    }

    uint8_t val = (static_cast<uint8_t>(cfg.mode2 == AmpMode::BYPASS) << 7)
                | (static_cast<uint8_t>(cfg.mode1 == AmpMode::BYPASS) << 6)
                | (cfg.attn_db & 0x3FU);

    reg stateReg;
    switch (state) {
        case AttSelect::STATE_A: stateReg = reg::STATE_A; break;
        case AttSelect::STATE_B: stateReg = reg::STATE_B; break;
        case AttSelect::STATE_C: stateReg = reg::STATE_C; break;
        case AttSelect::STATE_D: stateReg = reg::STATE_D; break;
        default: return Status::INVALID_ARG;
    }

    return writeReg(stateReg, val);
}

auto Driver::setDsaAttn(const AttSelect state, const uint8_t attn_db) noexcept -> Status {
    if (attn_db > MAX_DSA_ATTN) {
        return Status::INVALID_ATTN;
    }

    reg stateReg;
    switch (state) {
        case AttSelect::STATE_A: stateReg = reg::STATE_A; break;
        case AttSelect::STATE_B: stateReg = reg::STATE_B; break;
        case AttSelect::STATE_C: stateReg = reg::STATE_C; break;
        case AttSelect::STATE_D: stateReg = reg::STATE_D; break;
        default: return Status::INVALID_ARG;
    }

    auto [status, current] = readReg(stateReg);
    if (status != Status::OK) {
        return status;
    }

    uint8_t val = (current & 0xC0U) | (attn_db & 0x3FU);
    return writeReg(stateReg, val);
}

auto Driver::readActiveState() noexcept -> StateConfig {
    auto [status, val] = readReg(reg::ACTIVE_STATE_RDBK);
    if (status != Status::OK) {
        return {};
    }

    return StateConfig {
        .mode1   = (val & 0x40U) ? AmpMode::BYPASS : AmpMode::FIXED_GAIN,
        .mode2   = (val & 0x80U) ? AmpMode::BYPASS : AmpMode::FIXED_GAIN,
        .attn_db = static_cast<uint8_t>(val & 0x3FU),
    };
}

auto Driver::readFuses() noexcept -> std::array<uint8_t, 6> {
    std::array<uint8_t, 6> fuses {};

    const reg fuseRegs[6] = {
        reg::FUSE_RB0, reg::FUSE_RB1, reg::FUSE_RB2,
        reg::FUSE_RB3, reg::FUSE_RB4, reg::FUSE_RB5,
    };

    for (uint8_t i = 0; i < 6U; i++) {
        auto [status, val] = readReg(fuseRegs[i]);
        fuses[i] = (status == Status::OK) ? val : 0x00U;
    }

    return fuses;
}

auto Driver::writeReg(const reg r, const uint8_t data) noexcept -> Status {
    auto frame = buildFrame(true, r, data);

    auto status = _spi.transmit(frame.data(), SPI_NUM_BYTES);
    if (status != HAL_OK) {
        return Status::SPI_FAILURE;
    }

    return Status::OK;
};

auto Driver::readReg(const reg r) noexcept -> std::pair<Status, uint8_t> {
    auto transmitFrame = buildFrame(false, r, 0x00U);
    std::array<uint8_t, 1U> receiveFrame {};

    auto status = _spi.transceive(transmitFrame.data(), receiveFrame.data(), 1U);
    if (status != HAL_OK) {
        return std::make_pair(Status::SPI_FAILURE, 0U);
    }
    return std::make_pair(Status::OK, receiveFrame[0]);
};

auto Driver::buildFrame(const bool write, const reg r, const uint8_t data) noexcept -> std::array<uint8_t, SPI_NUM_BYTES> {
    std::array<uint8_t, SPI_NUM_BYTES> frame {0};

    const uint16_t addr = static_cast<uint16_t>(r);

    frame[0] = (write ? 0x00U : 0x80U) 
             | ((_config.chip_addr & 0x07U) << 4)
             | ((addr >> 8) & 0x01U);
    frame[1] = static_cast<uint8_t>(addr & 0xFFU);
    frame[2] = data;

    return frame;
};

}  // namespace amber::adl6331
