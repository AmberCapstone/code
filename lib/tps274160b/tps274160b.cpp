#include "tps274160b.hpp"

namespace amber::tps274160b {

Driver::Driver(Config& cfg) : _cfg(cfg) {}

auto Driver::enable(const uint8_t channel) noexcept -> void {
    if (channel >= kNumChannels) { return; }
    _cfg.enablePins[channel].SetHigh();
};

auto Driver::disable(const uint8_t channel) noexcept -> void {
    if (channel >= kNumChannels) { return; }
    _cfg.enablePins[channel].SetLow();
};

auto Driver::set(const uint8_t channel, bool enable) noexcept -> void {
    if (channel >= kNumChannels) { return; }
    _cfg.enablePins[channel].Set(enable);
};

auto Driver::enableAll() noexcept -> void {
    for (auto& pin : _cfg.enablePins) {
        pin.SetHigh();
    }
};

auto Driver::disableAll() noexcept -> void {
    for (auto& pin : _cfg.enablePins) {
        pin.SetLow();
    }
};

auto Driver::getFault() const noexcept -> bool {
    return _cfg.fault.Read();
};

auto Driver::diagEnable(bool en) noexcept -> void {
    _cfg.diagEn.Set(en);
};

auto Driver::selectDiagPin(uint8_t pin) noexcept -> void {
    if (pin >= kNumDiagPins) { return; }

    _cfg.diagSelect[0U].Set(pin & 0x1);
    _cfg.diagSelect[1U].Set((pin >> 1) & 0x1);
}

auto Driver::getCurrent() const noexcept -> float {
    const float voltage = _cfg.currentSense.ReadVoltage();
    return (voltage * kCurrentSenseRatio) / _cfg.currentSenseResistor;
};

}  // namespace amber::tps274160b
