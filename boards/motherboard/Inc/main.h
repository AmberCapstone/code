/* USER CODE BEGIN Header */
/**
 ******************************************************************************
 * @file           : main.h
 * @brief          : Header for main.c file.
 *                   This file contains the common defines of the application.
 ******************************************************************************
 * @attention
 *
 * Copyright (c) 2026 STMicroelectronics.
 * All rights reserved.
 *
 * This software is licensed under terms that can be found in the LICENSE file
 * in the root directory of this software component.
 * If no LICENSE file comes with this software, it is provided AS-IS.
 *
 ******************************************************************************
 */
/* USER CODE END Header */

/* Define to prevent recursive inclusion -------------------------------------*/
#ifndef __MAIN_H
#define __MAIN_H

#ifdef __cplusplus
extern "C" {
#endif

/* Includes ------------------------------------------------------------------*/
#include "stm32g0xx_hal.h"

/* Private includes ----------------------------------------------------------*/
/* USER CODE BEGIN Includes */

/* USER CODE END Includes */

/* Exported types ------------------------------------------------------------*/
/* USER CODE BEGIN ET */

/* USER CODE END ET */

/* Exported constants --------------------------------------------------------*/
/* USER CODE BEGIN EC */

/* USER CODE END EC */

/* Exported macro ------------------------------------------------------------*/
/* USER CODE BEGIN EM */

/* USER CODE END EM */

/* Exported functions prototypes ---------------------------------------------*/
void Error_Handler(void);

/* USER CODE BEGIN EFP */

/* USER CODE END EFP */

/* Private defines -----------------------------------------------------------*/
#define VGA_CS_N_Pin GPIO_PIN_4
#define VGA_CS_N_GPIO_Port GPIOE
#define TEMP_CS_N_Pin GPIO_PIN_5
#define TEMP_CS_N_GPIO_Port GPIOE
#define TEMP_ALERT_N_Pin GPIO_PIN_13
#define TEMP_ALERT_N_GPIO_Port GPIOC
#define P6V_SCATTER_PWR_EN_Pin GPIO_PIN_0
#define P6V_SCATTER_PWR_EN_GPIO_Port GPIOF
#define P6V_SCATTER_HSD_DIAG_EN_Pin GPIO_PIN_1
#define P6V_SCATTER_HSD_DIAG_EN_GPIO_Port GPIOF
#define LNA_EN_Pin GPIO_PIN_1
#define LNA_EN_GPIO_Port GPIOC
#define LOGAMP_EN_Pin GPIO_PIN_2
#define LOGAMP_EN_GPIO_Port GPIOC
#define LOG_VSENSE_Pin GPIO_PIN_0
#define LOG_VSENSE_GPIO_Port GPIOA
#define P6V_SCATTER_CS_Pin GPIO_PIN_4
#define P6V_SCATTER_CS_GPIO_Port GPIOA
#define DAC_ADJ_Pin GPIO_PIN_5
#define DAC_ADJ_GPIO_Port GPIOA
#define LPA_PWR_DET_Pin GPIO_PIN_7
#define LPA_PWR_DET_GPIO_Port GPIOA
#define COMPARATOR_Pin GPIO_PIN_5
#define COMPARATOR_GPIO_Port GPIOC
#define PWR_DOWN_Pin GPIO_PIN_0
#define PWR_DOWN_GPIO_Port GPIOB
#define PWR_DOWN_EXTI_IRQn EXTI0_1_IRQn
#define P6V_CS_TWO_Pin GPIO_PIN_1
#define P6V_CS_TWO_GPIO_Port GPIOB
#define P6V_CS_ONE_Pin GPIO_PIN_2
#define P6V_CS_ONE_GPIO_Port GPIOB
#define DEBUG2_Pin GPIO_PIN_7
#define DEBUG2_GPIO_Port GPIOE
#define DEBUG1_Pin GPIO_PIN_8
#define DEBUG1_GPIO_Port GPIOE
#define P12V_CS_Pin GPIO_PIN_10
#define P12V_CS_GPIO_Port GPIOB
#define FAN1_PWN_Pin GPIO_PIN_9
#define FAN1_PWN_GPIO_Port GPIOA
#define WARN_LIGHT_Pin GPIO_PIN_6
#define WARN_LIGHT_GPIO_Port GPIOC
#define FAN2_PWM_Pin GPIO_PIN_7
#define FAN2_PWM_GPIO_Port GPIOC
#define LPA_EN_Pin GPIO_PIN_12
#define LPA_EN_GPIO_Port GPIOD
#define VGA_ATTSEL0_Pin GPIO_PIN_13
#define VGA_ATTSEL0_GPIO_Port GPIOD
#define VGA_EN_Pin GPIO_PIN_14
#define VGA_EN_GPIO_Port GPIOD
#define VGA_ATTSEL1_Pin GPIO_PIN_15
#define VGA_ATTSEL1_GPIO_Port GPIOD
#define P6V_PG_Pin GPIO_PIN_8
#define P6V_PG_GPIO_Port GPIOF
#define USB_nFAULT_Pin GPIO_PIN_15
#define USB_nFAULT_GPIO_Port GPIOA
#define P5V_VSENSE_Pin GPIO_PIN_8
#define P5V_VSENSE_GPIO_Port GPIOC
#define P12V_VSENSE_Pin GPIO_PIN_9
#define P12V_VSENSE_GPIO_Port GPIOC
#define P12V_HSD_DIAG_EN_Pin GPIO_PIN_0
#define P12V_HSD_DIAG_EN_GPIO_Port GPIOD
#define MUX_ST_Pin GPIO_PIN_1
#define MUX_ST_GPIO_Port GPIOD
#define FAN1_PWR_EN_Pin GPIO_PIN_2
#define FAN1_PWR_EN_GPIO_Port GPIOD
#define nFAULT_FAN1_Pin GPIO_PIN_3
#define nFAULT_FAN1_GPIO_Port GPIOD
#define FAN2_PWR_EN_Pin GPIO_PIN_4
#define FAN2_PWR_EN_GPIO_Port GPIOD
#define nFAULT_FAN2_Pin GPIO_PIN_5
#define nFAULT_FAN2_GPIO_Port GPIOD
#define P6V_HDS_ONE_DIAG_EN_Pin GPIO_PIN_6
#define P6V_HDS_ONE_DIAG_EN_GPIO_Port GPIOD
#define P6V_HSD_ONE_nFAULT_Pin GPIO_PIN_7
#define P6V_HSD_ONE_nFAULT_GPIO_Port GPIOD
#define P6V_HSD_ONE_SEL_Pin GPIO_PIN_9
#define P6V_HSD_ONE_SEL_GPIO_Port GPIOF
#define P6V_HSD_ONE_SEH_Pin GPIO_PIN_10
#define P6V_HSD_ONE_SEH_GPIO_Port GPIOF
#define VGA_PWR_EN_Pin GPIO_PIN_11
#define VGA_PWR_EN_GPIO_Port GPIOF
#define VCO_PWR_EN_Pin GPIO_PIN_12
#define VCO_PWR_EN_GPIO_Port GPIOF
#define P6V_HSD_TWO_DIAG_EN_Pin GPIO_PIN_13
#define P6V_HSD_TWO_DIAG_EN_GPIO_Port GPIOF
#define P6V_HSD_TWO_nFAULT_Pin GPIO_PIN_3
#define P6V_HSD_TWO_nFAULT_GPIO_Port GPIOB
#define P6V_HSD_TWO_SEL_Pin GPIO_PIN_4
#define P6V_HSD_TWO_SEL_GPIO_Port GPIOB
#define P6V_HSD_TWO_SEH_Pin GPIO_PIN_5
#define P6V_HSD_TWO_SEH_GPIO_Port GPIOB
#define GEN_EN_Pin GPIO_PIN_0
#define GEN_EN_GPIO_Port GPIOE
#define LPA_PWR_EN_Pin GPIO_PIN_1
#define LPA_PWR_EN_GPIO_Port GPIOE
#define VCO_CE_Pin GPIO_PIN_2
#define VCO_CE_GPIO_Port GPIOE
#define VCO_LE_Pin GPIO_PIN_3
#define VCO_LE_GPIO_Port GPIOE
#define VCO_MUXOUT_Pin GPIO_PIN_6
#define VCO_MUXOUT_GPIO_Port GPIOB

/* USER CODE BEGIN Private defines */
extern int pwr_down_flag;
/* USER CODE END Private defines */

#ifdef __cplusplus
}
#endif

#endif /* __MAIN_H */
