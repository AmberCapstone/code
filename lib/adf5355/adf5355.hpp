/**
 * @file adf5355.hpp
 * @brief Driver for ADF5355 RF synthesizer
 */

#pragma once

#include <cstdint>
#include <algorithm>
#include <numeric>

#include "periph/spi.hpp"
#include "util/regbits.hpp"

extern "C" void HAL_Delay(uint32_t Delay);

namespace amber::adf5355 {

struct Driver {
    enum class DeviceId {
        ADF5355,
        ADF4355,
        ADF4355_2,
        ADF4355_3,
        ADF4356,
        ADF5356
    };

    enum class MuxOutSel {
        THREESTATE,
        DVDD,
        GND,
        R_DIV_OUT,
        N_DIV_OUT,
        ANALOG_LOCK_DETECT,
        DIGITAL_LOCK_DETECT
    };

    struct InitParam {
        DeviceId dev_id{DeviceId::ADF5355};
        uint64_t freq_req {5725000000ULL};
        uint8_t  freq_req_chan {0};
        uint32_t clkin_freq {100000000UL};
        uint32_t cp_ua {900};
        bool cp_neg_bleed_en {false};
        bool cp_gated_bleed_en {false};
        bool cp_bleed_current_polarity_en {false};
        bool mute_till_lock_en {false};
        bool outa_en {true};
        bool outb_en {false};
        uint8_t outa_power {0};
        uint8_t outb_power {0};
        bool phase_detector_polarity_neg {false};
        bool ref_diff_en {false};
        bool mux_out_3v3_en {true};
        uint8_t ref_doubler_en {0};
        uint8_t ref_div2_en {0};
        MuxOutSel mux_out_sel {MuxOutSel::DIGITAL_LOCK_DETECT};
        bool outb_sel_fund {false};
    };

    Driver(amber::periph::Spi& spi, const InitParam& param);
    ~Driver() = default;

    auto setup() noexcept -> int32_t;
    auto clk_recalc_rate(uint32_t chan, uint64_t& rate) const noexcept -> int32_t;
    auto clk_set_rate(uint32_t chan, uint64_t rate) noexcept -> int32_t;
    auto clk_round_rate(uint64_t rate, uint64_t& rounded_rate) const noexcept -> int32_t;

private:
    struct Reg0 {
        struct Config {
            using pos_t  = regbits::Pos<uint32_t, Config>;
            using mskd_t = regbits::Mskd<uint32_t, Config>;

            static constexpr pos_t INT_POS{4};
            static constexpr pos_t PRESCALER_POS{20};
            static constexpr pos_t AUTOCAL_POS{21};

            static constexpr uint32_t INT_MASK       = 0xFFFFU;
            static constexpr uint32_t PRESCALER_MASK = 0x1U;
            static constexpr uint32_t AUTOCAL_MASK   = 0x1U;
        };

        static auto build(uint32_t integer, bool prescaler, bool autocal) noexcept -> uint32_t;
    };

    struct Reg1 {
        struct Config {
            using pos_t  = regbits::Pos<uint32_t, Config>;
            using mskd_t = regbits::Mskd<uint32_t, Config>;

            static constexpr pos_t FRACT_POS{4};
            static constexpr uint32_t FRACT_MASK = 0xFFFFFFU;
        };

        static auto build(uint32_t fract1) noexcept -> uint32_t;
    };

    struct Reg2 {
        struct Config {
            using pos_t  = regbits::Pos<uint32_t, Config>;
            using mskd_t = regbits::Mskd<uint32_t, Config>;

            static constexpr pos_t MOD2_POS{4};
            static constexpr pos_t FRAC2_POS{18};
            static constexpr uint32_t MOD2_MASK  = 0x3FFFU;
            static constexpr uint32_t FRAC2_MASK = 0x3FFFU;
        };

        static auto build(uint32_t mod2, uint32_t fract2) noexcept -> uint32_t;
    };

    struct Reg6 {
        struct Config {
            using pos_t  = regbits::Pos<uint32_t, Config>;
            using mskd_t = regbits::Mskd<uint32_t, Config>;

            static constexpr pos_t OUTPUT_PWR_POS{4};
            static constexpr pos_t RF_OUT_EN_POS{6};
            static constexpr pos_t RF_OUTB_EN_POS{10};
            static constexpr pos_t MUTE_TILL_LOCK_EN_POS{11};
            static constexpr pos_t CP_BLEED_CURR_POS{13};
            static constexpr pos_t RF_DIV_SEL_POS{21};
            static constexpr pos_t FEEDBACK_FUND_POS{24};
            static constexpr pos_t NEG_BLEED_EN_POS{29};
            static constexpr pos_t GATED_BLEED_EN_POS{30};
            static constexpr pos_t BLEED_POLARITY_POS{31};

            static constexpr uint32_t OUTPUT_PWR_MASK = 0x3U;
            static constexpr uint32_t RF_OUT_EN_MASK = 0x1U;
            static constexpr uint32_t RF_OUTB_EN_MASK = 0x1U;
            static constexpr uint32_t CP_BLEED_CURR_MASK = 0xFFU;
            static constexpr uint32_t RF_DIV_SEL_MASK = 0x7U;
        };

        static auto build(
            uint8_t outa_power,
            bool outa_en,
            bool outb_en,
            bool mute_till_lock,
            uint8_t cp_bleed,
            uint8_t rf_div_sel,
            bool feedback_fund,
            bool neg_bleed,
            bool gated_bleed,
            bool bleed_polarity,
            bool is_adf4356_or_5356,
            bool outb_sel_fund
        ) noexcept -> uint32_t;
    };

    // Constants
    static constexpr uint64_t MIN_VCO_FREQ = 3400000000ULL;
    static constexpr uint64_t MAX_VCO_FREQ = 6800000000ULL;
    static constexpr uint64_t MAX_OUT_FREQ = MAX_VCO_FREQ;
    static constexpr uint64_t MIN_OUT_FREQ = MIN_VCO_FREQ / 64;
    static constexpr uint64_t MAX_OUTB_FREQ = MAX_VCO_FREQ * 2;
    static constexpr uint64_t MIN_OUTB_FREQ = MIN_VCO_FREQ * 2;

    static constexpr uint32_t MAX_FREQ_PFD = 75000000UL;
    static constexpr uint32_t MAX_MODULUS2 = 16384;
    static constexpr uint32_t MAX_MODULUS2_5356 = 268435456;
    static constexpr uint64_t MODULUS1 = 16777216ULL;
    static constexpr uint32_t MIN_INT_PRESCALER_89 = 75;

    static constexpr uint32_t REG5_DEFAULT = 0x00800025;
    static constexpr uint32_t REG8_DEFAULT_5355 = 0x102D0428;
    static constexpr uint32_t REG8_DEFAULT_4356_5356 = 0x15596568;
    static constexpr uint32_t REG11_DEFAULT = 0x0061300B;
    static constexpr uint32_t REG11_DEFAULT_4356_5356 = 0x0061200B;
    static constexpr uint32_t REG12_DEFAULT = 0x0000041C;
    static constexpr uint32_t REG12_DEFAULT_4356_5356 = 0x000005FC;

    amber::periph::Spi& spi_;
    DeviceId dev_id_;
    uint32_t regs[14]{};
    uint64_t freq_req_{0};
    uint8_t freq_req_chan_{0};
    uint8_t num_channels_{2};
    uint32_t clkin_freq_{0};
    uint64_t max_out_freq_{MAX_OUT_FREQ};
    uint64_t min_out_freq_{MIN_OUT_FREQ};
    uint64_t min_vco_freq_{MIN_VCO_FREQ};
    uint32_t fpfd_{0};
    uint32_t integer_{0};
    uint32_t fract1_{0};
    uint32_t fract2_{0};
    uint32_t mod2_{0};
    uint32_t cp_ua_{0};
    bool cp_neg_bleed_en_{false};
    bool cp_gated_bleed_en_{false};
    bool cp_bleed_current_polarity_en_{false};
    bool mute_till_lock_en_{false};
    bool outa_en_{true};
    bool outb_en_{false};
    uint8_t outa_power_{0};
    uint8_t outb_power_{0};
    bool phase_detector_polarity_neg_{false};
    bool ref_diff_en_{false};
    bool mux_out_3v3_en_{false};
    uint8_t ref_doubler_en_{0};
    uint8_t ref_div2_en_{0};
    bool outb_sel_fund_{false};
    uint8_t rf_div_sel_{0};
    uint16_t  ref_div_factor_{0};
    MuxOutSel mux_out_sel_{MuxOutSel::DIGITAL_LOCK_DETECT};
    bool all_synced_{false};

    static auto div_mod(uint64_t& dividend, uint64_t divisor) noexcept -> uint64_t;
    auto write(uint8_t reg_addr, uint32_t data) noexcept -> int32_t;

    static auto pll_fract_n_compute(
        uint64_t  vco,
        uint64_t  pfd,
        uint32_t& integer,
        uint32_t& fract1,
        uint32_t& fract2,
        uint32_t& mod2,
        uint32_t  max_modulus2
    ) noexcept -> void;

    auto pll_fract_n_get_rate(uint32_t channel) const noexcept -> uint64_t;
    auto reg_config(bool sync_all) noexcept -> int32_t;
    auto set_freq(uint64_t freq, uint8_t chan) noexcept -> int32_t;
};

} // namespace amber::devices
