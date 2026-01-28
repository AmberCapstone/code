#pragma once

#include <stdint.h>
#include <stdlib.h>

namespace amber::cobs {

constexpr size_t MaxEncodedLength(size_t raw_length) {
    constexpr size_t LEADING_ZERO = 1;
    constexpr size_t TERMINATING_ZERO = 1;
    size_t MAX_STUFF_BYTES = (raw_length + 253) / 254;
    return LEADING_ZERO + raw_length + MAX_STUFF_BYTES + TERMINATING_ZERO;
}

class Decoder {
public:
    Decoder(uint8_t* buffer);
    bool Decode(const uint8_t* encoded, size_t encoded_length);

    uint8_t* buffer;
    size_t length;

private:
    uint8_t block_remaining_;
    uint8_t code_;
};

size_t Encode(const uint8_t* raw, size_t length, uint8_t* output);

}  // namespace amber::cobs