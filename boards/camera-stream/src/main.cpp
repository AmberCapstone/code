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

typedef struct {
    ov7670::reg reg;
    uint8_t value;
} setting_t;

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

    // Synchronization pins

    // PCLK - rising edge indicates a new byte is available
    DDRD &= ~_BV(PD2);
    EICRA |= _BV(ISC01) | _BV(ISC00);
    // don't enable this interrupt yet - HREF handles it

    // VSYNC - rising edge indicates start of new frame
    DDRD &= ~_BV(PD3);
    EICRA |= _BV(ISC11) | _BV(ISC10);

    // HREF - rising edge indicates start of new row. stays high during the row
    DDRB &= ~_BV(PB0);
    PCICR |= _BV(PCIF0);

    twi::Initialize();
    serial::Initialize();

    sei();

    twi::WriteRegister(ov7670::reg::COM7, 0x80);
    // Wait at least 1 ms after reset (implementation guide 8.1.1)
    _delay_ms(2);

    constexpr setting_t settings[] = {
        // {ov7670::reg::CLKRC, 0x9f},  // input clock prescaler = 2
        // {ov7670::reg::TSLB, 0x00},   // output sequence YUYV

        // // VGA Settings (from Table 2-2)
        // {ov7670::reg::COM7, 0x00},   // VGA, YUV output (this is default)
        // {ov7670::reg::COM3, 0x00},   // No scale, no tristate (this is default)
        // {ov7670::reg::COM14, 0x00},  // No manual scaling, PCLK divider = 1
        // {ov7670::reg::COM10, 0x20},
        {ov7670::reg::SCALING_XSC, 0x3F},
        {ov7670::reg::SCALING_YSC, 0x3F},
        // {ov7670::reg::SCALING_DCWCTR, 0x11},
        // {ov7670::reg::SCALING_PCLK_DIV, 0xF0},
        // {ov7670::reg::SCALING_PCLK_DELAY, 0x02},
    };
    constexpr uint8_t LEN_SETTINGS = sizeof(settings) / sizeof(settings[0]);

    for (const setting_t* s = settings; s < settings + LEN_SETTINGS; s++) {
        twi::WriteRegister(s->reg, s->value);
    }

    // Enable pin interrupts
    EIMSK |= _BV(INT1) | _BV(INT0);
    PCMSK0 |= _BV(PCINT0);

    while (1) {
        continue;
    }
}

volatile bool y_byte = true;

ISR(PCINT0_vect) {
    if (PINB & _BV(PB0)) {
    } else {
        _delay_us(10);
        UDR0 = 0x00;    // send end-of-row delimiter
        y_byte = true;  // pattern is YUYV
    }
}

ISR(INT0_vect) {
    if (y_byte) {  // ignore U/V bytes
        uint8_t byte = (PIND & DATA_7_4_MASK) | (PINC & DATA_3_0_MASK);
        UDR0 = byte | 0x01;  // force != 0 to allow 0 as delimiter
    }
    y_byte = !y_byte;
}

ISR(INT1_vect) {
    // vsync
}