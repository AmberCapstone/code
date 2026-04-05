#include "tasks.hpp"

#include "FreeRTOS.h"
#include "dac.h"

#include "power/power.hpp"
#include "carrier/carrier.hpp"
#include "lib/adf5355/adf5355.hpp"
#include "lib/periph/analog_output.hpp"
#include "lib/periph/analog_input.hpp"
#include "lib/periph/digital.hpp"
#include "lib/periph/pwm.hpp"
#include "lib/periph/spi.hpp"
#include "main.h"
#include "spi.h"
#include "task.h"
#include "tim.h"
#include "dac.h"
#include "adc.h"

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

StaticTask_t t1hz_ctrl;
StackType_t t1hz_stack[STACK_SIZE_WORDS];

auto task_1000hz(void* argument) -> void {
    (void)argument;
    TickType_t wake_time = xTaskGetTickCount();

    // amber::periph::DigitalInput  comparator(*COMPARATOR_GPIO_Port, COMPARATOR_Pin);
    // amber::periph::AnalogOutput  logv(hdac1, DAC1_CHANNEL_2);
    // amber::periph::AnalogInput   read(hadc1, ADC_CHANNEL_0);
    // amber::periph::DigitalOutput debug2(*DEBUG2_GPIO_Port, DEBUG2_Pin);

    // // --- Configuration ---
    // constexpr uint32_t TASK_HZ      = 1000;
    // constexpr float    DECAY_RATE   = 0.001f;  // volts lost per tick — tune this
    // constexpr float    PEAK_FLOOR   = 0.0f;    // minimum peak hold value

    // float peak_hold = 0.0f;

    // logv.SetVoltage(0.95f);

    while (true) {
        vTaskDelayUntil(&wake_time, pdMS_TO_TICKS(1));

        // float new_sample = read.ReadVoltage();

        // // Latch any new peak immediately
        // if (new_sample > peak_hold) {
        //     peak_hold = new_sample;
        // } else {
        //     // Slowly decay when no new peak
        //     peak_hold = std::max(peak_hold - DECAY_RATE, PEAK_FLOOR);
        // }

        // debug2.Set(comparator.Read());
    }
}

auto task_100hz(void* argument) -> void {
    (void)argument;
    TickType_t wake_time = xTaskGetTickCount();

    while (true) {
        power::Update_100hz();
        carrier::Update_100hz();

        vTaskDelayUntil(&wake_time, pdMS_TO_TICKS(10));
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
    carrier::Init();

    xTaskCreateStatic(task_1hz, "1Hz", STACK_SIZE_WORDS, NULL, PRIORITY_1HZ,
                      t1hz_stack, &t1hz_ctrl);

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