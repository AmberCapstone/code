#include "twi.hpp"

#include "ov7670.hpp"

// avr
#include <avr/interrupt.h>
#include <avr/io.h>
#include <util/twi.h>

namespace twi {

static constexpr uint8_t MAX_ATTEMPTS = 4;

static inline uint8_t WaitStatus(void) {
    while ((TWCR & _BV(TWINT)) == 0);
    return TW_STATUS;
}

static inline uint8_t Start() {
    TWCR = _BV(TWINT) | _BV(TWSTA) | _BV(TWEN);  // Send start condition
    return WaitStatus();
}

static inline void Stop() {
    TWCR = _BV(TWINT) | _BV(TWSTO) | _BV(TWEN);
    while (TWCR & _BV(TWSTO));  // block until stop condition completes
}

static inline uint8_t Send(uint8_t data) {
    TWDR = data;                    // Load byte
    TWCR = _BV(TWINT) | _BV(TWEN);  // Clear prev interrupt and enable driver
    return WaitStatus();
}

void Initialize(void) {
    // F_SCL = F_CPU / (16 + 2 * TWBR * Prescaler)
    TWSR &= ~(_BV(TWPS1) | _BV(TWPS0));  // Prescaler = 1
    TWBR = 72;                           // F_SCL = 100 kHz
}

status_e WriteRegister(ov7670::reg reg, uint8_t data) {
    // Can simplify this by removing the ARB_LOST check since there is only one
    // master
    uint8_t twst = 0;
    status_e status = OK;
    uint8_t remaining_attempts = MAX_ATTEMPTS;

    while (true) {
        if (remaining_attempts-- == 0) {
            return ERROR;
        }

        twst = Start();
        if (twst == TW_MT_ARB_LOST) {
            continue;  // try again
        } else if (twst != TW_START && twst != TW_REP_START) {
            // Failed to start - don't try again, don't send stop condition
            return ERROR;
        }

        twst = Send(ov7670::WRITE_ADDRESS);
        if (twst == TW_MT_ARB_LOST) {
            continue;  // try again
        } else if (twst == TW_MT_SLA_ACK || twst == TW_MT_SLA_NACK) {
            status = OK;
        } else {
            status = ERROR;
            break;
        }

        twst = Send(static_cast<uint8_t>(reg));
        if (twst == TW_MT_ARB_LOST) {
            continue;
        } else if (twst == TW_MT_DATA_ACK || twst == TW_MT_DATA_NACK) {
            status = OK;
        } else {
            status = ERROR;
            break;
        }

        twst = Send(data);
        if (twst == TW_MT_ARB_LOST) {
            continue;
        } else if (twst == TW_MT_DATA_ACK || twst == TW_MT_DATA_NACK) {
            status = OK;
            break;
        } else {
            status = ERROR;
            break;
        }
    }

    Stop();
    return status;
}

status_e ReadRegister(ov7670::reg reg, uint8_t* out) {
    // Can simplify this by removing the ARB_LOST check since there is only one
    // master
    // A good structure might be:
    //  if(Start()) {
    //      TrySendBytes() // returns early if failure
    //      Stop()
    //  }
    // Is failure even possible? What are all the possible twst values after
    // each step?

    uint8_t twst = 0;
    status_e status = OK;
    uint8_t remaining_attempts = MAX_ATTEMPTS;

    while (true) {
        if (remaining_attempts-- == 0) {
            return ERROR;
        }

        twst = Start();
        if (twst == TW_MT_ARB_LOST) {
            continue;  // try again
        } else if (twst != TW_START && twst != TW_REP_START) {
            // Failed to start - don't try again, don't send stop condition
            return ERROR;
        }

        twst = Send(ov7670::WRITE_ADDRESS);
        if (twst == TW_MT_ARB_LOST) {
            continue;  // try again
        } else if (twst == TW_MT_SLA_ACK || twst == TW_MT_SLA_NACK) {
            status = OK;  // App Note 3.7.1 "The master continues... the
                          // following phases regardless of the response to the
                          // [9th] Don't Care bit by the slave(s)"
        } else {
            status = ERROR;
            break;
        }

        twst = Send(static_cast<uint8_t>(reg));
        if (twst == TW_MT_ARB_LOST) {
            continue;
        } else if (twst == TW_MT_DATA_ACK || twst == TW_MT_DATA_NACK) {
            status = OK;
            break;
        } else {
            status = ERROR;
            break;
        }
    }

    Stop();
    if (status != OK) {
        return status;
    }

    twst = Start();
    if (twst != TW_START && twst != TW_REP_START) {
        return ERROR;  // don't send stop condition
    }

    twst = Send(static_cast<uint8_t>(ov7670::READ_ADDRESS));
    if (twst != TW_MR_SLA_ACK && twst != TW_MR_SLA_NACK) {
        Stop();
        return ERROR;
    }

    TWCR = _BV(TWINT) | _BV(TWEN);
    twst = WaitStatus();
    if (twst != TW_MR_DATA_NACK) {
        Stop();
        return ERROR;
    }

    *out = TWDR;
    Stop();
    return OK;
}

}  // namespace twi