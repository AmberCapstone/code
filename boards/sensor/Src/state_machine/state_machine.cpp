#include "state_machine.hpp"

#include <cstdint>
#include <optional>

#include "flash.pb.h"
#include "flash/flash.hpp"
#include "sensor.pb.h"

namespace state_machine {

static uint32_t state_elapsed_ms = 0;

static std::optional<sensor_state_t> pending_transition = std::nullopt;
static sensor_state_t state = SENSOR_STATE_UNKNOWN;

static void Transition(sensor_state_t new_state) {
    pending_transition = new_state;
}

void Init(void) {
    Transition(SENSOR_STATE_IDLE);
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
        case SENSOR_STATE_UNKNOWN:  // Shouldn't ever get here
            break;

        case SENSOR_STATE_IDLE:
            break;

        case SENSOR_STATE_FLASHING:
            if (on_enter) {
                flash::Start();
            }

            if (flash::IsDone()) {
                Transition(SENSOR_STATE_IDLE);
            }
            break;

        case SENSOR_STATE_READOUT:
            if (on_enter) {
                flash::StartReadout();
            }

            if (flash::IsDone()) {
                Transition(SENSOR_STATE_IDLE);
            }
            break;
    }

    state_elapsed_ms += 1;
}

// Accessors
sensor_state_t GetState(void) {
    return state;
}

void PopulateStatus(sensor_status_t* msg) {
    msg->has_state = true;
    msg->state = state;
}

// Modifiers
void HandleAction(sensor_action_t action) {
    switch (action) {
        case SENSOR_ACTION_NONE:
            break;

        case SENSOR_ACTION_RESET:
            if (state != SENSOR_STATE_IDLE) {
                Transition(SENSOR_STATE_IDLE);
            }
            break;

        case SENSOR_ACTION_FLASH:
            if (state == SENSOR_STATE_IDLE) {
                Transition(SENSOR_STATE_FLASHING);
            }
            break;

        case SENSOR_ACTION_READOUT:
            if (state == SENSOR_STATE_IDLE) {
                Transition(SENSOR_STATE_READOUT);
            }
            break;
    }
}

}  // namespace state_machine