#include "cobs.hpp"

#if __has_include(<cstdint>)
#include <cstdint>
#else
#include <stdint.h>
#endif

namespace amber::cobs {

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
