#include "tasks.hpp"

#include "FreeRTOS.h"
#include "main.h"
#include "stm32g0xx_hal.h"
#include "task.h"

enum {
    PRIORITY_10HZ = 1,
    PRIORITY_100HZ = 2,
    PRIORITY_1000HZ = 3,
};

static const size_t STACK_SIZE_WORDS = 512;

StaticTask_t t1000hz_ctrl;
StackType_t t1000hz_stack[STACK_SIZE_WORDS];

StaticTask_t t100hz_ctrl;
StackType_t t100hz_stack[STACK_SIZE_WORDS];

StaticTask_t t10hz_ctrl;
StackType_t t10hz_stack[STACK_SIZE_WORDS];

void task_10hz(void* argument) {
    (void)argument;
    TickType_t wake_time = xTaskGetTickCount();

    while (true) {
        HAL_GPIO_TogglePin(DEBUG_GPIO_Port, DEBUG_Pin);
        vTaskDelayUntil(&wake_time, pdMS_TO_TICKS(100));
    }
};

void MX_FREERTOS_Init() {
    xTaskCreateStatic(task_10hz, "10Hz", STACK_SIZE_WORDS, NULL, PRIORITY_10HZ,
                      t10hz_stack, &t10hz_ctrl);

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