#include "cobs.hpp"

#if __has_include(<cstdint>)
#include <cstdint>
#else
#include <stdint.h>
#endif

namespace amber::cobs {

template <uint32_t BUF_SIZE>
Decoder<BUF_SIZE>::Decoder() {
    Reset();
}

template <uint32_t BUF_SIZE>
void Decoder<BUF_SIZE>::Reset() {
    length = 0;
    block_remaining_ = 0;
    code_ = 0xff;
}

template <uint32_t BUF_SIZE>
bool Decoder<BUF_SIZE>::Decode(const uint8_t* encoded,
                               uint32_t encoded_length) {
    // using `buf` as a cursor gives a ~10% speedup over `buffer[length++]`
    uint8_t* __restrict buf = buffer + length;

    // cache block_rem for a ~30% speedup
    uint8_t block_rem = block_remaining_;

    // caching `code` gives a 20% speedup for 256 0x00 bytes but a 5% slowdown
    // for 256 random bytes.
    // may remove this optimization later if actual data has few 0's
    uint8_t code = code_;

    for (const uint8_t* cursor = encoded; cursor < encoded + encoded_length;
         --block_rem) {
        uint8_t byte = *cursor++;
        if (byte == 0) {
            // COBS decoders usually only check byte==0 if block_rem==0
            // This hoisted check enables packet recovery but slows code by ~5%
            if (block_rem == 0) {
                // Expected byte=0. Regular exit path
                length = buf - buffer;
                block_remaining_ = block_rem;
                code_ = code;
                return true;
            } else {
                // block_remaining_ was incorrect
                // Reset to contain the error to this packet
                Reset();
                return false;
            }
        }
        if (block_rem > 0) {
            *buf++ = byte;
        } else {
            if (code != 0xff) {
                *buf++ = 0;
            }
            block_rem = byte;
            code = byte;
        }
    }
    code_ = code;
    length = buf - buffer;
    block_remaining_ = block_rem;
    return false;
}

}  // namespace amber::cobs