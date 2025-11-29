#include <Arduino.h>
#include <stdint.h>

#define WIDTH (640)
#define HEIGHT (480)

#define PACKET_SIZE (1 + WIDTH)

uint16_t cur_row = 0;
uint16_t x = 0;

uint8_t buffer[PACKET_SIZE] = {0};

void setup() {
    Serial.begin(2000000);
}

void loop() {
    for (uint16_t i = 0; i < WIDTH; i++) {
        buffer[i] = 255 * (x - i) * (x > i) * (x < i + 256);
        buffer[i] |= 0x01;
    }
    buffer[PACKET_SIZE - 1] = 0;  // end of row delimiter
    x = (x + 1) % WIDTH;
    cur_row = (cur_row + 1) % HEIGHT;
    Serial.write(buffer, PACKET_SIZE);
    _delay_ms(1);
}