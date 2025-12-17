#include "state_machine.hpp"

#include <cstdint>
#include <optional>

#include "spi_flash.pb.h"

namespace state_machine {

static uint32_t state_elapsed_ms = 0;

static std::optional<spi_flash_state_t> pending_transition = std::nullopt;
static spi_flash_state_t state = SPI_FLASH_STATE_UNKNOWN;

static void Transition(spi_flash_state_t new_state) {
    pending_transition = new_state;
}

void Init(void) {
    Transition(SPI_FLASH_STATE_IDLE);
}

void Update_1khz(void) {
    bool on_enter = false;

    if (pending_transition.has_value()) {
        on_enter = true;
        state = *pending_transition;
        state_elapsed_ms = 0;
        pending_transition.reset();
    }

    switch (state) {
        case SPI_FLASH_STATE_UNKNOWN:  // Shouldn't ever get here
            break;

        case SPI_FLASH_STATE_IDLE:
            break;

        case SPI_FLASH_STATE_FLASHING:
            if (state_elapsed_ms > 3000) {
                Transition(SPI_FLASH_STATE_IDLE);
            }
            break;

        case SPI_FLASH_STATE_READOUT:
            if (state_elapsed_ms > 5000) {
                Transition(SPI_FLASH_STATE_IDLE);
            }
            break;
    }

    state_elapsed_ms += 1;
}

// Accessors
void PopulateStatus(spi_flash_status_t* msg) {
    msg->has_state = true;
    msg->state = state;
}

// Modifiers
void HandleAction(spi_flash_action_t action) {
    switch (action) {
        case SPI_FLASH_ACTION_NONE:
            break;
        case SPI_FLASH_ACTION_FLASH:
            if (state == SPI_FLASH_STATE_IDLE) {
                Transition(SPI_FLASH_STATE_FLASHING);
            }
            break;
        case SPI_FLASH_ACTION_READOUT:
            if (state == SPI_FLASH_STATE_IDLE) {
                Transition(SPI_FLASH_STATE_READOUT);
            }
            break;
    }
}

}  // namespace state_machine