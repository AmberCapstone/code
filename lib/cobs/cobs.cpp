#include "cobs.hpp"

#include <stdint.h>
#include <stdlib.h>

namespace amber::cobs {

Decoder::Decoder(uint8_t* buffer) : buffer(buffer) {
    Reset();
}

void Decoder::Reset() {
    length = 0;
    block_remaining_ = 0;
    code_ = 0xff;
}

bool Decoder::Decode(const uint8_t* encoded, size_t encoded_length) {
    // adapted from
    // https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing
    // differences: supports procedural decoding / does not require `encoded` to
    // contain the entire message

    const uint8_t* input_cursor = encoded;

    for (; input_cursor < encoded + encoded_length; --block_remaining_) {
        if (block_remaining_ > 0) {
            uint8_t byte = *input_cursor++;
            if (byte == 0) {
                // block_remaining_ was incorrect. Reset to contain error to
                // this packet
                Reset();
                return false;
            }
            buffer[length++] = byte;
        } else {
            block_remaining_ = *input_cursor++;
            if (block_remaining_ == 0) {
                return true;
            }
            if (code_ != 0xff) {
                buffer[length++] = 0;
            }
            code_ = block_remaining_;
        }
    }
    return false;
}

size_t Encode(const uint8_t* raw, size_t length, uint8_t* output) {
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

    return size_t(encode_cursor - output);
}

}  // namespace amber::cobs
