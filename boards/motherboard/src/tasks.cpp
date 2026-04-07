#include "tasks.hpp"

#include "FreeRTOS.h"
#include "backscatter/backscatter.hpp"
#include "carrier/carrier.hpp"
#include "lib/periph/digital.hpp"
#include "main.h"
#include "power/power.hpp"
#include "serial/serial.hpp"
#include "task.h"
#include "thermal/thermal.hpp"
#include "tim.h"

enum {
    PRIORITY_1HZ = 1,
    PRIORITY_10HZ = 2,
    PRIORITY_100HZ = 3,
    PRIORITY_1000HZ = 4,
};

static const size_t STACK_SIZE_WORDS = 512;

StaticTask_t t1000hz_ctrl;
StackType_t t1000hz_stack[STACK_SIZE_WORDS];

StaticTask_t t100hz_ctrl;
StackType_t t100hz_stack[STACK_SIZE_WORDS];

StaticTask_t t10hz_ctrl;
StackType_t t10hz_stack[STACK_SIZE_WORDS];

StaticTask_t t1hz_ctrl;
StackType_t t1hz_stack[STACK_SIZE_WORDS];

auto task_1000hz(void* argument) -> void {
    (void)argument;
    TickType_t wake_time = xTaskGetTickCount();

    while (true) {
        serial::Receive();
        backscatter::Update1000hz();
        vTaskDelayUntil(&wake_time, pdMS_TO_TICKS(1));
    }
}

auto task_100hz(void* argument) -> void {
    (void)argument;
    TickType_t wake_time = xTaskGetTickCount();

    while (true) {
        power::Update_100hz();
        carrier::Update_100hz();
        serial::Update_100hz();

        vTaskDelayUntil(&wake_time, pdMS_TO_TICKS(10));
    }
};

auto task_10hz(void* argument) -> void {
    (void)argument;
    TickType_t wake_time = xTaskGetTickCount();

    while (true) {
        thermal::Update10Hz();

        vTaskDelayUntil(&wake_time, pdMS_TO_TICKS(100));
    }
};

auto task_1hz(void* argument) -> void {
    (void)argument;
    TickType_t wake_time = xTaskGetTickCount();

    amber::periph::DigitalOutput debug1(*DEBUG1_GPIO_Port, DEBUG1_Pin);

    while (true) {
        debug1.Toggle();

        vTaskDelayUntil(&wake_time, pdMS_TO_TICKS(1000));
    }
};

auto MX_FREERTOS_Init() -> void {
    power::Init();
    backscatter::Init();
    carrier::Init();
    thermal::Init();
    serial::Init();

    xTaskCreateStatic(task_1hz, "1Hz", STACK_SIZE_WORDS, NULL, PRIORITY_1HZ,
                      t1hz_stack, &t1hz_ctrl);

    xTaskCreateStatic(task_10hz, "10Hz", STACK_SIZE_WORDS, NULL, PRIORITY_10HZ,
                      t10hz_stack, &t10hz_ctrl);

    xTaskCreateStatic(task_100hz, "100Hz", STACK_SIZE_WORDS, NULL,
                      PRIORITY_100HZ, t100hz_stack, &t100hz_ctrl);

    xTaskCreateStatic(task_1000hz, "1000Hz", STACK_SIZE_WORDS, NULL,
                      PRIORITY_1000HZ, t1000hz_stack, &t1000hz_ctrl);

    vTaskStartScheduler();
}

/* static allocation is used for freeRTOS, so the application must
provide an implementation of vApplicationGetIdleTaskMemory() to provide
the memory that is used by the Idle task. */
extern "C" {
void vApplicationGetIdleTaskMemory(StaticTask_t** ppxIdleTaskTCBBuffer,
                                   StackType_t** ppxIdleTaskStackBuffer,
                                   uint32_t* pulIdleTaskStackSize) {
    static StaticTask_t xIdleTaskTCB;
    static StackType_t uxIdleTaskStack[configMINIMAL_STACK_SIZE];

    *ppxIdleTaskTCBBuffer = &xIdleTaskTCB;
    *ppxIdleTaskStackBuffer = uxIdleTaskStack;
    *pulIdleTaskStackSize = configMINIMAL_STACK_SIZE;
}
}
