#include "spi.hpp"

#include <avr/io.h>
#include <util/delay.h>

namespace amber::spi{

    static inline void cs_low() { PORTB &= ~_BV(PB2); }
    static inline void cs_high() { PORTB |= _BV(PB2); }

    static void set_clock_div(uint8_t div) {
        // div is mapped to SPR1, SPR0: SPI Clock Rate Select. 
        // SCK frequencies: f_osc/4 (default), /16, /64, /128
        // setting SPI2X doubles the speed

        bool spi2x, spr1, spr0;

        switch (div) {
            case 4:     spi2x = 0;  spr1 = 0;   spr0 = 0; break;
            case 16:    spi2x = 0;  spr1 = 0;   spr0 = 1; break;
            case 64:    spi2x = 0;  spr1 = 1;   spr0 = 0; break;
            case 128:   spi2x = 0;  spr1 = 1;   spr0 = 1; break;
            case 2:     spi2x = 1;  spr1 = 0;   spr0 = 0; break;
            case 8:     spi2x = 1;  spr1 = 0;   spr0 = 1; break;
            case 32:    spi2x = 1;  spr1 = 1;   spr0 = 0; break;
            default:    spi2x = 0;  spr1 = 0;   spr0 = 1; break;
        } //  from table 18-5 in ATmega328P_Datasheet

        // Clear and set SPR1 and SPR0 in SPCR - SPI Control Register
        SPCR &= ~(_BV(SPR1) | _BV(SPR0));
        if (spr0) SPCR |= _BV(SPR0);
        if (spr1) SPCR |= _BV(SPR1);

        // Set SPI2X in SPSR - SPI Status Register
        if (spi2x) SPSR |= _BV(SPI2X);
        else SPSR &= ~_BV(SPI2X);
    }

    void Initialize(const Config& cfg) {
        // SPI pins from Table 13-3 Port B Pins Alternate Functions and Arduino Datasheet
        // PB5    |    SCK    |    D13    |    output
        // PB4    |    MISO   |    D12    |    input
        // PB3    |    MOSI   |    D11    |    output
        // PB2    |    SS_n   |    D10    |    output

        DDRB |= _BV(DDB5) | _BV(DDB3) | _BV(DDB2); // outputs
        DDRB &= ~_BV(DDB4); // input

        cs_high();

        // set up SPCR - SPI Control Register
        uint8_t spcr = 0;
        spcr |= _BV(SPE); // Enable SPI
        if (cfg.master) spcr |= _BV(MSTR); 
        if (cfg.lsb_first) spcr |= _BV(DORD);
        switch (cfg.mode) {
            case Mode::Mode0: break;
            case Mode::Mode1: spcr |= _BV(CPHA); break;
            case Mode::Mode2: spcr |= _BV(CPOL); break;
            case Mode::Mode3: spcr |= _BV(CPOL) | _BV(CPHA); break;
        }
        SPCR = spcr;
        set_clock_div(cfg.clock_div);
        _delay_us(10);
    }

    uint8_t Transfer(uint8_t byte) {
        // SPI Data Register - used for data transfer bw reg file and SPI shift reg
        // Writing to the register initiates data transmission. 
        // Reading casues the shift register Receive buffer to be read. 
        SPDR = byte; 
        while (!(SPSR & _BV(SPIF))) {} // Wait for serial transfer complete
        return SPDR;
    }

    void CSAssert() {
        cs_low();
        _delay_us(2);
    }

    void CSDeassert() {
        _delay_us(2);
        cs_high();
    }

    void PacketTransfer(const uint8_t* tx, uint8_t* rx, const uint16_t len) {
        CSAssert();
        for (uint16_t i = 0; i < len; i++) {
            rx[i] = Transfer(tx[i]);
        }
        CSDeassert();
    }

} // namespace amber::spi
