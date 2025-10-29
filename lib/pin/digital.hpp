#pragma once

namespace pin {

struct DigitalOutput {
    virtual void setHigh() = 0;
    virtual void setLow() = 0;
    virtual ~DigitalOutput() = default;
};

struct DigitalInput {
    virtual bool read() const = 0;
    virtual ~DigitalInput() = default;
};

} // namespace pin
