/* USER CODE BEGIN Header */
/**
 ******************************************************************************
 * @file           : main.h
 * @brief          : Header for main.c file.
 *                   This file contains the common defines of the application.
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

/* Define to prevent recursive inclusion -------------------------------------*/
#ifndef __MAIN_H
#define __MAIN_H

#ifdef __cplusplus
extern "C" {
#endif

/* Includes ------------------------------------------------------------------*/
#include "stm32u0xx_hal.h"

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
#define CAM_PWRDN_Pin GPIO_PIN_13
#define CAM_PWRDN_GPIO_Port GPIOC
#define CAM_RESETn_Pin GPIO_PIN_3
#define CAM_RESETn_GPIO_Port GPIOC
#define ISENSE_Pin GPIO_PIN_0
#define ISENSE_GPIO_Port GPIOA
#define FPGA_ISENSE_Pin GPIO_PIN_1
#define FPGA_ISENSE_GPIO_Port GPIOA
#define VSENSE_Pin GPIO_PIN_4
#define VSENSE_GPIO_Port GPIOA
#define DEBUG_LED_Pin GPIO_PIN_5
#define DEBUG_LED_GPIO_Port GPIOA
#define CAM_PWR_EN_Pin GPIO_PIN_6
#define CAM_PWR_EN_GPIO_Port GPIOA
#define FPGA_PWR_EN_Pin GPIO_PIN_7
#define FPGA_PWR_EN_GPIO_Port GPIOA
#define FPGA_GPIO2_Pin GPIO_PIN_5
#define FPGA_GPIO2_GPIO_Port GPIOC
#define FPGA_GPIO1_Pin GPIO_PIN_0
#define FPGA_GPIO1_GPIO_Port GPIOB
#define FPGA_PWRDN_Pin GPIO_PIN_1
#define FPGA_PWRDN_GPIO_Port GPIOB
#define FPGA_DRDY_Pin GPIO_PIN_2
#define FPGA_DRDY_GPIO_Port GPIOB
#define FPGA_CDONE_Pin GPIO_PIN_10
#define FPGA_CDONE_GPIO_Port GPIOB
#define FPGA_CRESETn_Pin GPIO_PIN_11
#define FPGA_CRESETn_GPIO_Port GPIOB
#define FPGA_CSn_Pin GPIO_PIN_12
#define FPGA_CSn_GPIO_Port GPIOB
#define FPGA_SCK_Pin GPIO_PIN_13
#define FPGA_SCK_GPIO_Port GPIOB
#define FPGA_MISO_Pin GPIO_PIN_14
#define FPGA_MISO_GPIO_Port GPIOB
#define FPGA_MOSI_Pin GPIO_PIN_15
#define FPGA_MOSI_GPIO_Port GPIOB
#define VBAT_OK_Pin GPIO_PIN_9
#define VBAT_OK_GPIO_Port GPIOA
#define USB_PWR_ON_Pin GPIO_PIN_10
#define USB_PWR_ON_GPIO_Port GPIOA
#define FLASH_SCK_Pin GPIO_PIN_10
#define FLASH_SCK_GPIO_Port GPIOC
#define FLASH_MISO_Pin GPIO_PIN_11
#define FLASH_MISO_GPIO_Port GPIOC
#define FLASH_MOSI_Pin GPIO_PIN_12
#define FLASH_MOSI_GPIO_Port GPIOC
#define FLASH_CSn_Pin GPIO_PIN_4
#define FLASH_CSn_GPIO_Port GPIOB
#define FLASH_RESETn_Pin GPIO_PIN_5
#define FLASH_RESETn_GPIO_Port GPIOB
#define GPIO1_Pin GPIO_PIN_8
#define GPIO1_GPIO_Port GPIOB

/* USER CODE BEGIN Private defines */

/* USER CODE END Private defines */

#ifdef __cplusplus
}
#endif

#endif /* __MAIN_H */
