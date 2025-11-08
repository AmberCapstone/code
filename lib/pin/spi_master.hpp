#pragma once

#include <stdint.h>
#include <stdlib.h>

namespace amber {

class SpiMaster {
public:
    virtual void Transmit(uint8_t* tx_data, uint32_t length);
    virtual void Receive(uint8_t* rx_data, uint32_t length);
    virtual void TransmitReceive(uint8_t* tx_data, uint8_t* rx_data,
                                 uint32_t length);
    virtual void SetChipSelect(bool high);
};

}  // namespace amber