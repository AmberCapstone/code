/* USER CODE BEGIN Header */
/**
 ******************************************************************************
 * @file    stm32u0xx_it.c
 * @brief   Interrupt Service Routines.
 ******************************************************************************
 * @attention
 *
 * Copyright (c) 2025 STMicroelectronics.
 * All rights reserved.
 *
 * This software is licensed under terms that can be found in the LICENSE file
 * in the root directory of this software component.
 * If no LICENSE file comes with this software, it is provided AS-IS.
 *
 ******************************************************************************
 */
/* USER CODE END Header */

/* Includes ------------------------------------------------------------------*/
#include "stm32u0xx_it.h"

#include "FreeRTOS.h"
#include "main.h"
#include "task.h"
/* Private includes ----------------------------------------------------------*/
/* USER CODE BEGIN Includes */
/* USER CODE END Includes */

/* Private typedef -----------------------------------------------------------*/
/* USER CODE BEGIN TD */

/* USER CODE END TD */

/* Private define ------------------------------------------------------------*/
/* USER CODE BEGIN PD */

/* USER CODE END PD */

/* Private macro -------------------------------------------------------------*/
/* USER CODE BEGIN PM */

/* USER CODE END PM */

/* Private variables ---------------------------------------------------------*/
/* USER CODE BEGIN PV */

/* USER CODE END PV */

/* Private function prototypes -----------------------------------------------*/
/* USER CODE BEGIN PFP */

/* USER CODE END PFP */

/* Private user code ---------------------------------------------------------*/
/* USER CODE BEGIN 0 */

/* USER CODE END 0 */

/* External variables --------------------------------------------------------*/
extern DMA_HandleTypeDef hdma_adc1;
extern PCD_HandleTypeDef hpcd_USB_DRD_FS;
/* USER CODE BEGIN EV */

/* USER CODE END EV */

/******************************************************************************/
/*           Cortex Processor Interruption and Exception Handlers          */
/******************************************************************************/
/**
 * @brief This function handles Non maskable interrupt.
 */
void NMI_Handler(void) {
    /* USER CODE BEGIN NonMaskableInt_IRQn 0 */

    /* USER CODE END NonMaskableInt_IRQn 0 */
    /* USER CODE BEGIN NonMaskableInt_IRQn 1 */
    while (1) {
    }
    /* USER CODE END NonMaskableInt_IRQn 1 */
}

/**
 * @brief This function handles Hard fault interrupt.
 */
void HardFault_Handler(void) {
    /* USER CODE BEGIN HardFault_IRQn 0 */

    /* USER CODE END HardFault_IRQn 0 */
    while (1) {
        /* USER CODE BEGIN W1_HardFault_IRQn 0 */
        /* USER CODE END W1_HardFault_IRQn 0 */
    }
}

/**
 * @brief This function handles System tick timer.
 */
void SysTick_Handler(void) {
    /* USER CODE BEGIN SysTick_IRQn 0 */

    /* USER CODE END SysTick_IRQn 0 */
    HAL_IncTick();
#if (INCLUDE_xTaskGetSchedulerState == 1)
    if (xTaskGetSchedulerState() != taskSCHEDULER_NOT_STARTED) {
#endif /* INCLUDE_xTaskGetSchedulerState */
        xPortSysTickHandler();
#if (INCLUDE_xTaskGetSchedulerState == 1)
    }
#endif /* INCLUDE_xTaskGetSchedulerState */
       /* USER CODE BEGIN SysTick_IRQn 1 */

    /* USER CODE END SysTick_IRQn 1 */
}

/******************************************************************************/
/* STM32U0xx Peripheral Interrupt Handlers                                    */
/* Add here the Interrupt Handlers for the used peripherals.                  */
/* For the available peripheral interrupt handler names,                      */
/* please refer to the startup file (startup_stm32u0xx.s).                    */
/******************************************************************************/

/**
 * @brief This function handles USB FS global interrupt.
 */
void USB_DRD_FS_IRQHandler(void) {
    /* USER CODE BEGIN USB_DRD_FS_IRQn 0 */

    /* USER CODE END USB_DRD_FS_IRQn 0 */
    HAL_PCD_IRQHandler(&hpcd_USB_DRD_FS);
    /* USER CODE BEGIN USB_DRD_FS_IRQn 1 */

    /* USER CODE END USB_DRD_FS_IRQn 1 */
}

/**
 * @brief This function handles DMA1 channel 1 interrupt.
 */
void DMA1_Channel1_IRQHandler(void) {
    /* USER CODE BEGIN DMA1_Channel1_IRQn 0 */

    /* USER CODE END DMA1_Channel1_IRQn 0 */
    HAL_DMA_IRQHandler(&hdma_adc1);
    /* USER CODE BEGIN DMA1_Channel1_IRQn 1 */

    /* USER CODE END DMA1_Channel1_IRQn 1 */
}

/**
 * @brief This function handles DMAMUX_OVR_IT + DMA1 channel 4 to 7 + DMA2
 * channel 1 to 5.
 */
void DMA1_Ch4_7_DMA2_Ch1_5_DMAMUX_OVR_IRQHandler(void) {
    /* USER CODE BEGIN DMA1_Ch4_7_DMA2_Ch1_5_DMAMUX_OVR_IRQn 0 */

    /* USER CODE END DMA1_Ch4_7_DMA2_Ch1_5_DMAMUX_OVR_IRQn 0 */
    // Handle DMAMUX
    // Handle DMA1_Channel1
    HAL_DMAEx_MUX_IRQHandler(&hdma_adc1);
    /* USER CODE BEGIN DMA1_Ch4_7_DMA2_Ch1_5_DMAMUX_OVR_IRQn 1 */

    /* USER CODE END DMA1_Ch4_7_DMA2_Ch1_5_DMAMUX_OVR_IRQn 1 */
}

/* USER CODE BEGIN 1 */

/* USER CODE END 1 */
