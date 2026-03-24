#include "adf5355.hpp"

namespace amber::adf5355 {

auto Driver::Reg0::build(uint32_t integer, bool prescaler, bool autocal) noexcept -> uint32_t {
    return (
        Config::mskd_t(Config::INT_MASK, integer, Config::INT_POS) |
        Config::mskd_t(Config::PRESCALER_MASK, prescaler, Config::PRESCALER_POS)|
        Config::mskd_t(Config::AUTOCAL_MASK, autocal, Config::AUTOCAL_POS)).bits();
}

auto Driver::Reg1::build(uint32_t fract1) noexcept -> uint32_t {
    return Config::mskd_t(Config::FRACT_MASK, fract1, Config::FRACT_POS).bits();
}

auto Driver::Reg2::build(uint32_t mod2, uint32_t fract2) noexcept -> uint32_t {
    const auto mod  = Config::mskd_t(Config::MOD2_MASK,  mod2,  Config::MOD2_POS);
    const auto frac = Config::mskd_t(Config::FRAC2_MASK, fract2, Config::FRAC2_POS);
    return (mod | frac).bits();
}

auto Driver::Reg6::build(
    uint8_t  outa_power,
    bool     outa_en,
    bool     outb_en,
    bool     mute_till_lock,
    uint8_t  cp_bleed,
    uint8_t  rf_div_sel,
    bool     feedback_fund,
    bool     neg_bleed,
    bool     gated_bleed,
    bool     bleed_polarity,
    bool     is_adf4356_or_5356,
    bool     outb_sel_fund
) noexcept -> uint32_t {
    uint32_t v = 0;

    v |= Config::mskd_t(Config::OUTPUT_PWR_MASK, outa_power, Config::OUTPUT_PWR_POS).bits();
    v |= Config::mskd_t(Config::RF_OUT_EN_MASK, outa_en, Config::RF_OUT_EN_POS).bits();
    v |= Config::mskd_t(Config::RF_OUTB_EN_MASK, outb_en, Config::RF_OUTB_EN_POS).bits();
    v |= Config::mskd_t(0x1U, mute_till_lock, Config::MUTE_TILL_LOCK_EN_POS).bits();
    v |= Config::mskd_t(Config::CP_BLEED_CURR_MASK, cp_bleed, Config::CP_BLEED_CURR_POS).bits();
    v |= Config::mskd_t(Config::RF_DIV_SEL_MASK, rf_div_sel, Config::RF_DIV_SEL_POS).bits();
    v |= Config::mskd_t(0x1U, feedback_fund, Config::FEEDBACK_FUND_POS).bits();
    v |= Config::mskd_t(0x1U, neg_bleed, Config::NEG_BLEED_EN_POS).bits();
    v |= Config::mskd_t(0x1U, gated_bleed, Config::GATED_BLEED_EN_POS).bits();

    if (is_adf4356_or_5356) {
        v |= Config::mskd_t(0x1U, bleed_polarity, Config::BLEED_POLARITY_POS).bits();
        v |= regbits::Mskd<uint32_t, Config>(0x1U, outb_sel_fund, regbits::Pos<uint32_t, Config>(25)).bits();
    }

    return v;
}

auto Driver::div_mod(uint64_t& dividend, uint64_t divisor) noexcept -> uint64_t {
    uint64_t rem = dividend % divisor;
    dividend /= divisor;
    return rem;
}

auto Driver::write(uint8_t reg_addr, uint32_t data) noexcept -> int32_t {
    uint8_t buf[4];
    uint32_t full = data | reg_addr;
    buf[0] = static_cast<uint8_t>(full >> 24);
    buf[1] = static_cast<uint8_t>(full >> 16);
    buf[2] = static_cast<uint8_t>(full >> 8);
    buf[3] = static_cast<uint8_t>(full);
    return (spi_.transmit(buf, 4) == HAL_OK) ? 0 : -1;
}

auto Driver::pll_fract_n_compute(
    uint64_t  vco,
    uint64_t  pfd,
    uint32_t& integer,
    uint32_t& fract1,
    uint32_t& fract2,
    uint32_t& mod2,
    uint32_t  max_modulus2
) noexcept -> void {
    uint64_t tmp = div_mod(vco, pfd);
    tmp = tmp * MODULUS1;
    fract2 = static_cast<uint32_t>(div_mod(tmp, pfd));
    integer = static_cast<uint32_t>(vco);
    fract1 = static_cast<uint32_t>(tmp);
    mod2 = static_cast<uint32_t>(pfd);

    while (mod2 > max_modulus2) {
        mod2 >>= 1;
        fract2 >>= 1;
    }

    uint32_t gcd = std::gcd(fract2, mod2);
    mod2 /= gcd;
    fract2 /= gcd;
}

auto Driver::pll_fract_n_get_rate(uint32_t channel) const noexcept -> uint64_t {
    uint64_t val = (static_cast<uint64_t>(integer_) * MODULUS1 + fract1_) * fpfd_;
    uint64_t tmp = static_cast<uint64_t>(fract2_) * fpfd_;
    div_mod(tmp, mod2_);
    val += tmp + (MODULUS1 / 2);
    val /= (MODULUS1 * (1ULL << (channel == 1 ? 0 : rf_div_sel_)));
    if (channel == 1) val <<= 1;
    return val;
}

auto Driver::reg_config(bool sync_all) noexcept -> int32_t {
    uint32_t max_reg = (dev_id_ == DeviceId::ADF4356 || dev_id_ == DeviceId::ADF5356) ? 13 : 12;

    if (sync_all || !all_synced_) {
        for (int32_t i = max_reg; i >= 1; --i) {
            if (write(static_cast<uint8_t>(i), regs[i]) != 0) return -1;
        }
        all_synced_ = true;
    } else {
        if (dev_id_ == DeviceId::ADF4356 || dev_id_ == DeviceId::ADF5356) {
            if (write(13, regs[13]) != 0) return -1;
        }
        if (write(10, regs[10]) != 0) return -1;
        if (write(6, regs[6]) != 0) return -1;
        if (write(4, regs[4] | (1U << 4)) != 0) return -1;
        if (write(2, regs[2]) != 0) return -1;
        if (write(1, regs[1]) != 0) return -1;
        if (write(0, regs[0] & ~(1U << 21)) != 0) return -1;
        if (write(4, regs[4]) != 0) return -1;
    }

    HAL_Delay(20);

    return write(0, regs[0]);
}

auto Driver::set_freq(uint64_t freq, uint8_t chan) noexcept -> int32_t {
    if (chan >= num_channels_) return -1;

    bool is_adf4356 = (dev_id_ == DeviceId::ADF4356);

    if (chan == 0) {
        if (freq > max_out_freq_ || freq < min_out_freq_) return -1;
        rf_div_sel_ = 0;
        outa_en_ = true;
        while (freq < min_vco_freq_) {
            freq <<= 1;
            rf_div_sel_++;
        }
    } else if (is_adf4356) {
        if (freq > max_out_freq_ || freq < min_out_freq_ || !outb_sel_fund_) return -1;
        rf_div_sel_ = 0;
        outb_en_ = true;
        while (freq < min_vco_freq_) {
            freq <<= 1;
            rf_div_sel_++;
        }
    } else {
        if (freq > MAX_OUTB_FREQ || freq < MIN_OUTB_FREQ) return -1;
        outb_en_ = true;
        freq >>= 1;
    }

    const uint32_t max_mod = (dev_id_ == DeviceId::ADF4356 || dev_id_ == DeviceId::ADF5356)
                             ? MAX_MODULUS2_5356 : MAX_MODULUS2;

    pll_fract_n_compute(freq, fpfd_, integer_, fract1_, fract2_, mod2_, max_mod);

    const bool prescaler = (integer_ >= MIN_INT_PRESCALER_89);

    bool cp_neg_bleed = (fpfd_ > 100000000UL || (fract1_ == 0 && fract2_ == 0))
                        ? false : cp_neg_bleed_en_;

    uint32_t cp_bleed;
    if (dev_id_ == DeviceId::ADF4356 || dev_id_ == DeviceId::ADF5356) {
        cp_bleed = (24U * (fpfd_ / 1000U) * cp_ua_) / (61440U * 900U);
    } else {
        cp_bleed = ((400U * cp_ua_) + (integer_ * 375U) - 1U) / (integer_ * 375U);
    }
    cp_bleed = std::clamp(cp_bleed, 1UL, 255UL);

    regs[0]  = Reg0::build(integer_, prescaler, true);
    regs[1]  = Reg1::build(fract1_);
    regs[2]  = Reg2::build(mod2_, fract2_);

    if (dev_id_ == DeviceId::ADF4356 || dev_id_ == DeviceId::ADF5356) {
        regs[13] = ((mod2_ >> 14) << 4) | ((fract2_ >> 14) << 18);
    }

    regs[6] = Reg6::build(
        outa_power_, outa_en_, outb_en_, mute_till_lock_en_,
        static_cast<uint8_t>(cp_bleed), rf_div_sel_, true,
        cp_neg_bleed, cp_gated_bleed_en_, cp_bleed_current_polarity_en_,
        (dev_id_ == DeviceId::ADF4356 || dev_id_ == DeviceId::ADF5356),
        outb_sel_fund_
    );

    freq_req_ = freq;
    return reg_config(all_synced_);
}

auto Driver::setup() noexcept -> int32_t {
    ref_div_factor_ = 0;
    do {
        ref_div_factor_++;
        fpfd_ = (clkin_freq_ * (ref_doubler_en_ ? 2U : 1U)) /
                (ref_div_factor_ * (ref_div2_en_ ? 2U : 1U));
    } while (fpfd_ > MAX_FREQ_PFD);

    uint32_t tmp = std::clamp(static_cast<uint32_t>((cp_ua_ - 315U) / 315U), 0UL, 15UL);

    regs[4] = (0U << 4) | (0U << 5) | (0U << 6) |
              ((!phase_detector_polarity_neg_) << 7) |
              (mux_out_3v3_en_ << 8) |
              (ref_diff_en_ << 9) |
              (tmp << 10) |
              (1U << 14) |
              (ref_div_factor_ << 15) |
              (ref_div2_en_ << 25) |
              (ref_doubler_en_ << 26) |
              (static_cast<uint32_t>(mux_out_sel_) << 27);

    regs[5] = REG5_DEFAULT;

    regs[7] = (0U << 4) | (3U << 5) | (0U << 7) | (0U << 8) | (1U << 25) |
              ((dev_id_ == DeviceId::ADF5356) ? 0x04000007U : 0x10000007U);

    regs[8] = ((dev_id_ == DeviceId::ADF4356 || dev_id_ == DeviceId::ADF5356)
               ? REG8_DEFAULT_4356_5356 : REG8_DEFAULT_5355);

    tmp = std::clamp(((fpfd_ + 599999U) / 600000U), 1UL, 1023UL);
    regs[9] = (tmp << 14) |
              (((fpfd_ * 2U + 99999U) / (100000U * tmp)) << 4) |
              (((fpfd_ * 5U + 99999U) / (100000U * tmp)) << 9) |
              (((fpfd_ + ((dev_id_ == DeviceId::ADF4356 || dev_id_ == DeviceId::ADF5356) ? 1599999U : 2399999U)) /
                ((dev_id_ == DeviceId::ADF4356 || dev_id_ == DeviceId::ADF5356) ? 1600000U : 2400000U)) << 24);

    tmp = std::clamp(((fpfd_ / 100000U) - 2U) / 4U, 1UL, 255UL);

    regs[10] = (1U << 4) | (1U << 5) | (tmp << 6) | 0x00C0000A;

    regs[11] = (dev_id_ == DeviceId::ADF4356 || dev_id_ == DeviceId::ADF5356)
               ? REG11_DEFAULT_4356_5356 : REG11_DEFAULT;

    regs[12] = ((dev_id_ == DeviceId::ADF4356 || dev_id_ == DeviceId::ADF5356)
                ? (1U << 12) | REG12_DEFAULT_4356_5356
                : (1U << 16) | REG12_DEFAULT);

    all_synced_ = false;

    return set_freq(freq_req_, freq_req_chan_);
}

Driver::Driver(amber::periph::Spi& spi, const InitParam& param)
    : spi_(spi)
    , dev_id_(param.dev_id)
    , freq_req_(param.freq_req)
    , freq_req_chan_(param.freq_req_chan)
    , clkin_freq_(param.clkin_freq)
    , cp_ua_(param.cp_ua)
    , cp_neg_bleed_en_(param.cp_neg_bleed_en)
    , cp_gated_bleed_en_(param.cp_gated_bleed_en)
    , cp_bleed_current_polarity_en_(param.cp_bleed_current_polarity_en)
    , mute_till_lock_en_(param.mute_till_lock_en)
    , outa_en_(param.outa_en)
    , outb_en_(param.outb_en)
    , outa_power_(param.outa_power)
    , outb_power_(param.outb_power)
    , phase_detector_polarity_neg_(param.phase_detector_polarity_neg)
    , ref_diff_en_(param.ref_diff_en)
    , mux_out_3v3_en_(param.mux_out_3v3_en)
    , ref_doubler_en_(param.ref_doubler_en)
    , ref_div2_en_(param.ref_div2_en)
    , outb_sel_fund_(param.outb_sel_fund)
    , mux_out_sel_(param.mux_out_sel)
{
    switch (dev_id_) {
        case DeviceId::ADF4356:
        case DeviceId::ADF5356:
        case DeviceId::ADF5355:
            max_out_freq_ = MAX_OUT_FREQ;
            min_out_freq_ = MIN_OUT_FREQ;
            min_vco_freq_ = MIN_VCO_FREQ;
            break;
        case DeviceId::ADF4355:
            max_out_freq_ = MAX_OUT_FREQ;
            min_out_freq_ = MIN_OUT_FREQ;
            min_vco_freq_ = 3400000000ULL;
            break;
        case DeviceId::ADF4355_2:
            max_out_freq_ = 4400000000ULL;
            min_out_freq_ = 3400000000ULL / 64;
            min_vco_freq_ = 3400000000ULL;
            break;
        case DeviceId::ADF4355_3:
            max_out_freq_ = 6600000000ULL;
            min_out_freq_ = 3300000000ULL / 64;
            min_vco_freq_ = 3300000000ULL;
            break;
    }
}

auto Driver::clk_recalc_rate(uint32_t chan, uint64_t& rate) const noexcept -> int32_t {
    if (chan >= num_channels_) return -1;
    rate = pll_fract_n_get_rate(chan);
    return 0;
}

auto Driver::clk_set_rate(uint32_t chan, uint64_t rate) noexcept -> int32_t {
    if (chan >= num_channels_) return -1;
    return set_freq(rate, static_cast<uint8_t>(chan));
}

auto Driver::clk_round_rate(uint64_t rate, uint64_t& rounded_rate) const noexcept -> int32_t {
    rounded_rate = rate;
    return 0;
}

} // namespace amber::adf5355
