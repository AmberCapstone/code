#pragma once

#if __has_include(<cstdint>)
#include <cstdint>
#else
#include <stdint.h>
#endif

namespace amber::cobs {

constexpr uint32_t MaxEncodedLength(uint32_t raw_length) {
    constexpr uint32_t LEADING_ZERO = 1;
    constexpr uint32_t TERMINATING_ZERO = 1;
    uint32_t MAX_STUFF_BYTES = (raw_length + 253) / 254;
    return LEADING_ZERO + raw_length + MAX_STUFF_BYTES + TERMINATING_ZERO;
}

template <uint32_t BUF_SIZE>
class Decoder {
public:
    Decoder();

    bool Decode(const uint8_t* encoded, uint32_t encoded_length);

    void Reset(void);

    uint32_t length;

private:
    uint8_t block_remaining_;
    uint8_t code_;

public:
    uint8_t buffer[BUF_SIZE];
};

uint32_t Encode(const uint8_t* raw, uint32_t length, uint8_t* output);

}  // namespace amber::cobs

#include "cobs.tpp"