#pragma once

#if __has_include(<cstdint>)
#include <cstdint>
#else
#include <stdint.h>
#endif

namespace amber {

class SpiMaster {
public:
    virtual void Transmit(uint8_t* tx_data, uint32_t length) = 0;
    virtual void Receive(uint8_t* rx_data, uint32_t length) = 0;
    virtual void TransmitReceive(uint8_t* tx_data, uint8_t* rx_data,
                                 uint32_t length) = 0;
    virtual void SetChipSelect(bool high) = 0;
};

}  // namespace amber
