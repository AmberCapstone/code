#include "tasks.hpp"

#include <stdbool.h>

#include "cobs.hpp"
#include "common_macros.hpp"
#include "m25pe.hpp"
#include "spi_master.hpp"

// Submodules
#include "flash/flash.hpp"
#include "sensors/sensors.hpp"
#include "serial/serial.hpp"
#include "state_machine/state_machine.hpp"

// FreeRTOS
#include "FreeRTOS.h"
#include "task.h"

enum {
    PRIORITY_1000HZ = 3,
    PRIORITY_10HZ = 1,
};

static const size_t STACK_SIZE_WORDS = 512;

StaticTask_t t1000hz_ctrl;
StackType_t t1000hz_stack[STACK_SIZE_WORDS];

StaticTask_t t10hz_ctrl;
StackType_t t10hz_stack[STACK_SIZE_WORDS];

void task_1000hz(void* argument) {
    (void)argument;
    TickType_t wake_time = xTaskGetTickCount();

    while (true) {
        serial::Receive();
        state_machine::Update_1khz();

        xTaskDelayUntil(&wake_time, pdMS_TO_TICKS(1));
    }
}

void task_10hz(void* argument) {
    (void)argument;
    TickType_t wake_time = xTaskGetTickCount();

    while (true) {
        sensors::Update_10hz();
        serial::SendStatus();

        HAL_GPIO_TogglePin(LD4_GPIO_Port, LD4_Pin);
        xTaskDelayUntil(&wake_time, pdMS_TO_TICKS(100));
    }
}

void MX_FREERTOS_Init() {
    serial::Init();
    flash::Init();
    sensors::Init();
    state_machine::Init();

    xTaskCreateStatic(task_1000hz, "1000Hz", STACK_SIZE_WORDS, NULL,
                      PRIORITY_1000HZ, t1000hz_stack, &t1000hz_ctrl);

    xTaskCreateStatic(task_10hz, "10Hz", STACK_SIZE_WORDS, NULL, PRIORITY_10HZ,
                      t10hz_stack, &t10hz_ctrl);

    vTaskStartScheduler();
}