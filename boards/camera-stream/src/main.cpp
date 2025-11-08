#include <Arduino.h>
#include <stdint.h>

#include "cobs.hpp"

#define WIDTH (640)
#define HEIGHT (480)

#define PACKET_SIZE (2 + WIDTH)

uint16_t cur_row = 0;
uint16_t x = 0;

uint8_t buffer[PACKET_SIZE] = {0};
uint8_t cobs_buffer[amber::cobs::MaxEncodedLength(PACKET_SIZE)];

void setup() {
    Serial.begin(1000000);
}

void loop() {
    buffer[0] = cur_row & 0xff;
    buffer[1] = cur_row >> 8;
    for (uint16_t i = 0; i < WIDTH; i++) {
        buffer[2 + i] = 255 * (x - i) * (x > i) * (x < i + 256);
    }
    x = (x + 2) % WIDTH;
    cur_row = (cur_row + 1) % HEIGHT;
    size_t len = amber::cobs::Encode(buffer, PACKET_SIZE, cobs_buffer);
    Serial.write(cobs_buffer, len);
}