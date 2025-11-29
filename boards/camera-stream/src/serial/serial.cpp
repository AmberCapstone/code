#include "serial.hpp"

#include "avr/interrupt.h"
#include "avr/io.h"
#include "cobs.hpp"

using namespace amber;

namespace serial {

static uint8_t tx_buffer[amber::cobs::MaxEncodedLength(MAX_LENGTH)];
static uint16_t tx_count;
static uint16_t tx_index;

void Initialize(void) {
    // BAUD = F_CPU / (8 * UBRR) - 1
    UBRR0H = 0;
    UBRR0L = 0;           // 0 = 2M, 1 = 1M, 3 = 500K, 7 = 250K
    UCSR0A |= _BV(U2X0);  // 2x speed asynchronous UART

    // enable TX and RX (also configures pins)
    UCSR0B |= _BV(RXEN0) | _BV(TXEN0);

    // Async UART, no parity, 1 stop bit, 8-bit characters, sample on falling
    // edge
    UCSR0C |= _BV(UCSZ01) | _BV(UCSZ00);  // 8-bit
}

void Transmit(uint8_t* buffer, uint16_t length) {
    tx_count = cobs::Encode(buffer, length, tx_buffer);

    // Even empty inputs have encoeded length >= 1
    UDR0 = tx_buffer[0];
    tx_index = 1;

    UCSR0B |= _BV(UDRIE0);  // enable interrupt
}

ISR(USART_UDRE_vect) {
    if (tx_index < tx_count) {
        UDR0 = tx_buffer[tx_index++];
    } else {
        // transmission is complete
        UCSR0B &= ~_BV(UDRIE0);  // disable interrupt
    }
}

}  // namespace serial
