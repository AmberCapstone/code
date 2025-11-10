#include <avr/interrupt.h>
#include <avr/io.h>
#include <stdint.h>
#include <util/twi.h>

#define DATA_3_0_MASK (_BV(PD3) | _BV(PD2) | _BV(PD1) | _BV(PD0))
#define DATA_7_4_MASK (_BV(PD7) | _BV(PD6) | _BV(PD5) | _BV(PD4))

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
    DDRD &= ~_BV(PD3);  // VSYNC - rising edge indicates start of new frame

    // TWI (I2C) setup
    DDRC |= _BV(PC4);  // SDA output
    DDRC |= _BV(PC5);  // SCL output
    // F_SCL = F_CPU / (16 + 2 * TWBR * Prescaler)
    TWSR &= ~(_BV(TWPS1) | _BV(TWPS0));  // Prescaler = 1
    TWBR = 72;                           // F_SCL = 100 kHz

    // Serial setup
    // BAUD = F_CPU / (8 * UBRR) - 1
    UBRR0H = 0;
    UBRR0L = 7;           // 0 = 2M, 1 = 1M, 3 = 500K, 7 = 250K
    UCSR0A |= _BV(U2X0);  // 2x speed asynchronous UART

    // enable TX and RX (also configures pins)
    UCSR0B |= _BV(RXEN0) | _BV(TXEN0);

    // Async UART, no parity, 1 stop bit, 8-bit characters, sample on falling
    // edge
    UCSR0C |= _BV(UCSZ01) | _BV(UCSZ00);  // 8-bit

    while (1) {
    }
}