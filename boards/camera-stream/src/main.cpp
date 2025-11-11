#include <stdio.h>

#include "ov7670.hpp"

// avr
#include <avr/interrupt.h>
#include <avr/io.h>
#include <avr/pgmspace.h>
#include <stdint.h>
#include <util/delay.h>

// modules
#include "serial/serial.hpp"
#include "twi/twi.hpp"

#define DATA_3_0_MASK (_BV(PD3) | _BV(PD2) | _BV(PD1) | _BV(PD0))
#define DATA_7_4_MASK (_BV(PD7) | _BV(PD6) | _BV(PD5) | _BV(PD4))

using namespace amber;

volatile uint16_t rows = 0;
volatile uint16_t frames = 0;

typedef struct {
    ov7670::reg reg;
    uint8_t value;
} setting_t;

const setting_t settings[] = {
    {ov7670::reg::TSLB, 0x04},  // output sequence YUYV

    // VGA Settings (from Table 2-2)
    {ov7670::reg::CLKRC, 0x01},  // input clock prescaler = 2
    // {COM7, 0x00},  // VGA, YUV output (this is default)
    // {COM3, 0x00}, // No scale, no tristate (this is default)
    {ov7670::reg::COM14, 0x00},  // No manual scaling, PCLK divider = 1
    {ov7670::reg::SCALING_XSC, 0x3A},
    {ov7670::reg::SCALING_YSC, 0x35},
    {ov7670::reg::SCALING_DCWCTR, 0x11},
    {ov7670::reg::SCALING_PCLK_DIV, 0xF0},
    {ov7670::reg::SCALING_PCLK_DELAY, 0x02},
};

const uint8_t LEN_SETTINGS = sizeof(settings) / sizeof(settings[0]);

int main(void) {
    cli();

    // init 8MHz pixel clock.
    // uses 2-bit PWM cycle. output on PB3 = OC2A
    DDRB |= _BV(PB3);
    TCCR2A = _BV(COM2A0) | _BV(WGM21) | _BV(WGM20);
    TCCR2B = _BV(WGM22) | _BV(CS20);
    OCR2A = 0;  // freq = fcpu / (2* (OCR2A+1))

    // configure DATA[3:0] and DATA[7:4] as inputs
    DDRC &= ~DATA_3_0_MASK;
    DDRD &= ~DATA_7_4_MASK;

    // Interupt pins
    DDRD &= ~_BV(PD2);  // HREF - rising edge indicates start of new row
    EICRA |= _BV(ISC01) | _BV(ISC00);

    DDRD &= ~_BV(PD3);  // VSYNC - rising edge indicates start of new frame
    EICRA |= _BV(ISC11) | _BV(ISC10);
    EIMSK |= _BV(INT1) | _BV(INT0);

    twi::Initialize();
    serial::Initialize();

    sei();

    twi::WriteRegister(ov7670::reg::COM7, 0x80);
    // Wait at least 1 ms after reset (implementation guide 8.1.1)
    _delay_ms(2);

    // for (const setting_t* s = settings; s < settings + LEN_SETTINGS; s++)
    // {
    //     twi::WriteRegister(settings->reg, settings->value);
    // }

    char buffer[100];
    uint8_t pid = 0;
    uint8_t ver = 0;

    while (1) {
        twi::ReadRegister(ov7670::reg::VER, &ver);
        twi::ReadRegister(ov7670::reg::PID, &pid);

        // uint16_t len = sprintf(buffer, "V: %d\t H: %d\n", frames, rows);
        uint16_t len = sprintf(buffer, "PID: %02x\t VER: %02x", pid, ver);
        serial::Transmit((uint8_t*)buffer, len);
        _delay_ms(1000);
    }
}

ISR(INT0_vect) {
    rows += 1;
}

ISR(INT1_vect) {
    frames += 1;
}