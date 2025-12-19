#include "cobs.hpp"

#if __has_include(<cstdint>)
#include <cstdint>
#else
#include <stdint.h>
#endif

namespace amber::cobs {

Decoder::Decoder(uint8_t* buffer) : buffer(buffer) {
    Reset();
}

void Decoder::Reset() {
    length = 0;
    block_remaining_ = 0;
    code_ = 0xff;
}

bool Decoder::Decode(const uint8_t* encoded, uint32_t encoded_length) {
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

uint32_t Encode(const uint8_t* raw, uint32_t length, uint8_t* output) {
    uint8_t* encode_cursor = output;

    uint8_t zero_offset = 1;
    uint8_t* zero_offset_p = encode_cursor++;

    for (const uint8_t* byte = raw; length--; ++byte) {
        if (*byte != 0) {
            *encode_cursor++ = *byte;
            ++zero_offset;
        }
        if ((*byte == 0) || (zero_offset == 0xff)) {
            *zero_offset_p = zero_offset;
            zero_offset = 1;
            zero_offset_p = encode_cursor;
            if ((*byte == 0) || (length > 0)) {
                ++encode_cursor;
            }
        }
    }
    *zero_offset_p = zero_offset;  // write the final zero_offset value
    *encode_cursor++ = 0;          // write delimiter

    return uint32_t(encode_cursor - output);
}

}  // namespace amber::cobs
