#include "serial.hpp"

#include <avr/interrupt.h>
#include <avr/io.h>
#include "cobs.hpp"

using namespace amber;

namespace serial {

static constexpr uint16_t RX_MAX_ENCODED = cobs::MaxEncodedLength(MAX_LENGTH);

static uint8_t tx_buffer[RX_MAX_ENCODED];
static volatile uint16_t tx_count = 0;
static volatile uint16_t tx_index = 0;
static volatile bool busy = false;

static volatile uint16_t rx_count = 0;
static volatile bool packet_ready = false;
static volatile bool rx_overflow = false;
static uint8_t rx_buffer[RX_MAX_ENCODED];

static uint8_t decoded_buffer[MAX_LENGTH];

bool IsBusy(void) { return busy; }
bool PacketReady(void) { return packet_ready; }

void Initialize(void) {
    // BAUD = F_CPU / (8 * UBRR) - 1
    UBRR0H = 0;
    UBRR0L = 3;           // 0 = 2M, 1 = 1M, 3 = 500K, 7 = 250K
    UCSR0A |= _BV(U2X0);  // 2x speed asynchronous UART

    // enable TX and RX (also configures pins), and enable interrupt on RX complete
    UCSR0B |= _BV(RXEN0) | _BV(TXEN0) | _BV(RXCIE0);

    // Async UART, no parity, 1 stop bit, 8-bit characters, sample on falling
    // edge
    UCSR0C |= _BV(UCSZ01) | _BV(UCSZ00);  // 8-bit
}

void Transmit(const uint8_t* buffer, uint16_t length) {
    if (busy) return;
    busy = true; 

    tx_count = cobs::Encode(buffer, length, tx_buffer);

    // Even empty inputs have encoeded length >= 1
    UDR0 = tx_buffer[0];
    tx_index = 1;

    UCSR0B |= _BV(UDRIE0);  // enable interrupt
}

uint16_t ReadPacket(uint8_t* out) {
    if (!packet_ready) return 0;

    static uint8_t encoded[RX_MAX_ENCODED];
    uint16_t len = 0;

    UCSR0B &= ~_BV(RXCIE0);
    len = rx_count;
    if (len > RX_MAX_ENCODED) len = RX_MAX_ENCODED;

    for (uint16_t i = 0; i < len; i++) {
        encoded[i] = rx_buffer[i];
    }

    rx_count = 0;
    packet_ready = false;
    rx_overflow = false;
    UCSR0B |= _BV(RXCIE0);

    cobs::Decoder dec(decoded_buffer);
    bool done = dec.Decode(encoded, len);
    if (!done) return 0;

    uint16_t n = (uint16_t)dec.length;
    if (n > MAX_LENGTH) n = MAX_LENGTH;

    for (uint16_t i = 0; i < n; i ++) {
        out[i] = decoded_buffer[i];
    }
    return n;
}

ISR(USART_UDRE_vect) { // data register empty interrupt vector
    if (tx_index < tx_count) {
        UDR0 = tx_buffer[tx_index++];
    } else {
        // transmission is complete
        UCSR0B &= ~_BV(UDRIE0);  // disable interrupt
        busy = false;
    }
}

ISR(USART_RX_vect) { // RX Complete interrupt vector
    uint8_t byte = UDR0; // Read received data. This should clear the RXC0 flag so a new interrupt will not occur once the ISR terminates

    if (packet_ready) {
        // if (byte == 0x00) {
        //     packet_ready = false;
        //     rx_count = 0;
        // }
        return;
    }

    if (rx_overflow) {
        if (byte == 0x00) {
            rx_overflow = false;
            rx_count = 0;
        }
        return;
    }

    if (rx_count < RX_MAX_ENCODED) {
        rx_buffer[rx_count++] = byte;
        if (byte == 0x00) {
            packet_ready = true;
        }
    } else {
        rx_overflow = true; 
    }
}

}  // namespace serial
