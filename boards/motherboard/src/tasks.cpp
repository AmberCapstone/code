#include "tasks.hpp"

#include "FreeRTOS.h"
#include "lib/adf5355/adf5355.hpp"
#include "lib/periph/digital.hpp"
#include "lib/periph/pwm.hpp"
#include "lib/periph/spi.hpp"
#include "main.h"
#include "spi.h"
#include "task.h"
#include "tim.h"

// the current state of this file is for lab
// testing - it will be severly refactored soon

enum {
    PRIORITY_1HZ = 1,
    PRIORITY_10HZ = 2,
    PRIORITY_100HZ = 3,
    PRIORITY_1000HZ = 4,
};

static const size_t STACK_SIZE_WORDS = 512;

StaticTask_t t100hz_ctrl;
StackType_t t100hz_stack[STACK_SIZE_WORDS];

StaticTask_t t1hz_ctrl;
StackType_t t1hz_stack[STACK_SIZE_WORDS];

auto task_100hz(void* argument) -> void {
    (void)argument;
    TickType_t wake_time = xTaskGetTickCount();

    // GPIOs
    amber::periph::DigitalOutput vco_pwr_en(*VCO_PWR_EN_GPIO_Port,
                                            VCO_PWR_EN_Pin);
    amber::periph::DigitalOutput gen_en(*GEN_EN_GPIO_Port, GEN_EN_Pin);
    amber::periph::DigitalOutput vga_pwr_en(*VGA_PWR_EN_GPIO_Port,
                                            VGA_PWR_EN_Pin);
    amber::periph::DigitalOutput lpa_pwr_en(*LPA_PWR_EN_GPIO_Port,
                                            LPA_PWR_EN_Pin);
    amber::periph::DigitalOutput p6v_scatter_pwr_en(
        *P6V_SCATTER_PWR_EN_GPIO_Port, P6V_SCATTER_PWR_EN_Pin);
    amber::periph::DigitalOutput lpa_en(*LPA_EN_GPIO_Port, LPA_EN_Pin);
    amber::periph::DigitalOutput lna_en(*LNA_EN_GPIO_Port, LNA_EN_Pin);
    amber::periph::DigitalOutput p6v_scatter_hsd_diag_en(
        *P6V_SCATTER_HSD_DIAG_EN_GPIO_Port, P6V_SCATTER_HSD_DIAG_EN_Pin);
    amber::periph::DigitalOutput logamp_en(*LOGAMP_EN_GPIO_Port, LOGAMP_EN_Pin);
    amber::periph::DigitalOutput fan2_pwr_en(*FAN2_PWR_EN_GPIO_Port,
                                             FAN2_PWR_EN_Pin);
    amber::periph::DigitalOutput fan1_pwr_en(*FAN1_PWR_EN_GPIO_Port,
                                             FAN1_PWR_EN_Pin);
    amber::periph::DigitalOutput fan1_pwm(*FAN1_PWN_GPIO_Port, FAN1_PWN_Pin);
    amber::periph::DigitalOutput warn_light(*WARN_LIGHT_GPIO_Port,
                                            WARN_LIGHT_Pin);
    amber::periph::DigitalOutput vco_le(*VCO_LE_GPIO_Port, VCO_LE_Pin);

    amber::periph::Spi spi2(hspi2, vco_le);
    amber::periph::Pwm fan2_pwm(htim2, TIM_CHANNEL_4);

    // initialize the ADF5355
    amber::adf5355::Driver::InitParam param{};
    param.freq_req = 5725000000ULL;
    param.clkin_freq = 100000000UL;
    amber::adf5355::Driver adf(spi2, param);
    adf.setup();

    fan2_pwm.Start();
    fan2_pwm.SetDutyCycle(100.0f);

    vco_pwr_en.SetHigh();
    gen_en.SetHigh();
    vga_pwr_en.SetHigh();
    lpa_pwr_en.SetHigh();
    p6v_scatter_pwr_en.SetHigh();
    lpa_en.SetHigh();
    lna_en.SetHigh();
    p6v_scatter_hsd_diag_en.SetHigh();
    logamp_en.SetHigh();
    fan2_pwr_en.SetHigh();
    fan1_pwr_en.SetHigh();
    fan1_pwm.SetHigh();
    warn_light.SetHigh();

    while (true) {
        if (pwr_down_flag) {
            vco_pwr_en.SetLow();
            gen_en.SetLow();
            vga_pwr_en.SetLow();
            lpa_pwr_en.SetLow();
            p6v_scatter_pwr_en.SetLow();
            lpa_en.SetLow();
            lna_en.SetLow();
            p6v_scatter_hsd_diag_en.SetLow();
            logamp_en.SetLow();
            fan2_pwr_en.SetLow();
            fan1_pwr_en.SetLow();
            fan1_pwm.SetLow();
            warn_light.SetLow();

            warn_light.SetLow();
            break;
        }

        vTaskDelayUntil(&wake_time, pdMS_TO_TICKS(10));
    }
};

auto task_1hz(void* argument) -> void {
    amber::periph::DigitalOutput debug1(*DEBUG1_GPIO_Port, DEBUG1_Pin);

    while (true) {
        debug1.Toggle();

        vTaskDelay(pdMS_TO_TICKS(1000));
    }
};

auto MX_FREERTOS_Init() -> void {
    xTaskCreateStatic(task_1hz, "1Hz", STACK_SIZE_WORDS, NULL, PRIORITY_1HZ,
                      t1hz_stack, &t1hz_ctrl);

    xTaskCreateStatic(task_100hz, "100Hz", STACK_SIZE_WORDS, NULL,
                      PRIORITY_100HZ, t100hz_stack, &t100hz_ctrl);

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