/* USER CODE BEGIN Header */
/**
 ******************************************************************************
 * @file    gpio.c
 * @brief   This file provides code for the configuration
 *          of all used GPIO pins.
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

/* Includes ------------------------------------------------------------------*/
#include "gpio.h"

/* USER CODE BEGIN 0 */

/* USER CODE END 0 */

/*----------------------------------------------------------------------------*/
/* Configure GPIO                                                             */
/*----------------------------------------------------------------------------*/
/* USER CODE BEGIN 1 */

/* USER CODE END 1 */

/** Configure pins
 */
void MX_GPIO_Init(void) {
    GPIO_InitTypeDef GPIO_InitStruct = {0};

    /* GPIO Ports Clock Enable */
    __HAL_RCC_GPIOC_CLK_ENABLE();
    __HAL_RCC_GPIOE_CLK_ENABLE();
    __HAL_RCC_GPIOF_CLK_ENABLE();
    __HAL_RCC_GPIOA_CLK_ENABLE();
    __HAL_RCC_GPIOB_CLK_ENABLE();
    __HAL_RCC_GPIOD_CLK_ENABLE();

    /*Configure GPIO pin Output Level */
    HAL_GPIO_WritePin(GPIOE,
                      VGA_CS_N_Pin | TEMP_CS_N_Pin | DEBUG2_Pin | DEBUG1_Pin |
                          GEN_EN_Pin | LPA_PWR_EN_Pin | VCO_CE_Pin | VCO_LE_Pin,
                      GPIO_PIN_RESET);

    /*Configure GPIO pin Output Level */
    HAL_GPIO_WritePin(GPIOF,
                      P6V_SCATTER_PWR_EN_Pin | P6V_SCATTER_HSD_DIAG_EN_Pin |
                          P6V_HSD_ONE_SEL_Pin | P6V_HSD_ONE_SEH_Pin |
                          VGA_PWR_EN_Pin | VCO_PWR_EN_Pin |
                          P6V_HSD_TWO_DIAG_EN_Pin,
                      GPIO_PIN_RESET);

    /*Configure GPIO pin Output Level */
    HAL_GPIO_WritePin(GPIOC, LNA_EN_Pin | LOGAMP_EN_Pin | WARN_LIGHT_Pin,
                      GPIO_PIN_RESET);

    /*Configure GPIO pin Output Level */
    HAL_GPIO_WritePin(FAN1_PWN_GPIO_Port, FAN1_PWN_Pin, GPIO_PIN_RESET);

    /*Configure GPIO pin Output Level */
    HAL_GPIO_WritePin(GPIOD,
                      LPA_EN_Pin | VGA_ATTSEL0_Pin | VGA_EN_Pin |
                          VGA_ATTSEL1_Pin | P12V_HSD_DIAG_EN_Pin |
                          FAN1_PWR_EN_Pin | FAN2_PWR_EN_Pin |
                          P6V_HDS_ONE_DIAG_EN_Pin,
                      GPIO_PIN_RESET);

    /*Configure GPIO pin Output Level */
    HAL_GPIO_WritePin(GPIOB, P6V_HSD_TWO_SEL_Pin | P6V_HSD_TWO_SEH_Pin,
                      GPIO_PIN_RESET);

    /*Configure GPIO pins : VGA_CS_N_Pin TEMP_CS_N_Pin */
    GPIO_InitStruct.Pin = VGA_CS_N_Pin | TEMP_CS_N_Pin;
    GPIO_InitStruct.Mode = GPIO_MODE_OUTPUT_OD;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    GPIO_InitStruct.Speed = GPIO_SPEED_FREQ_LOW;
    HAL_GPIO_Init(GPIOE, &GPIO_InitStruct);

    /*Configure GPIO pins : TEMP_ALERT_N_Pin P5V_VSENSE_Pin P12V_VSENSE_Pin */
    GPIO_InitStruct.Pin = TEMP_ALERT_N_Pin | P5V_VSENSE_Pin | P12V_VSENSE_Pin;
    GPIO_InitStruct.Mode = GPIO_MODE_INPUT;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    HAL_GPIO_Init(GPIOC, &GPIO_InitStruct);

    /*Configure GPIO pins : P6V_SCATTER_PWR_EN_Pin P6V_SCATTER_HSD_DIAG_EN_Pin
       P6V_HSD_ONE_SEL_Pin P6V_HSD_ONE_SEH_Pin VGA_PWR_EN_Pin VCO_PWR_EN_Pin
       P6V_HSD_TWO_DIAG_EN_Pin */
    GPIO_InitStruct.Pin = P6V_SCATTER_PWR_EN_Pin | P6V_SCATTER_HSD_DIAG_EN_Pin |
                          P6V_HSD_ONE_SEL_Pin | P6V_HSD_ONE_SEH_Pin |
                          VGA_PWR_EN_Pin | VCO_PWR_EN_Pin |
                          P6V_HSD_TWO_DIAG_EN_Pin;
    GPIO_InitStruct.Mode = GPIO_MODE_OUTPUT_PP;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    GPIO_InitStruct.Speed = GPIO_SPEED_FREQ_LOW;
    HAL_GPIO_Init(GPIOF, &GPIO_InitStruct);

    /*Configure GPIO pins : LNA_EN_Pin LOGAMP_EN_Pin WARN_LIGHT_Pin */
    GPIO_InitStruct.Pin = LNA_EN_Pin | LOGAMP_EN_Pin | WARN_LIGHT_Pin;
    GPIO_InitStruct.Mode = GPIO_MODE_OUTPUT_PP;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    GPIO_InitStruct.Speed = GPIO_SPEED_FREQ_LOW;
    HAL_GPIO_Init(GPIOC, &GPIO_InitStruct);

    /*Configure GPIO pin : PWR_DOWN_Pin */
    GPIO_InitStruct.Pin = PWR_DOWN_Pin;
    GPIO_InitStruct.Mode = GPIO_MODE_IT_RISING;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    HAL_GPIO_Init(PWR_DOWN_GPIO_Port, &GPIO_InitStruct);

    /*Configure GPIO pins : DEBUG2_Pin DEBUG1_Pin GEN_EN_Pin LPA_PWR_EN_Pin
                             VCO_CE_Pin VCO_LE_Pin */
    GPIO_InitStruct.Pin = DEBUG2_Pin | DEBUG1_Pin | GEN_EN_Pin |
                          LPA_PWR_EN_Pin | VCO_CE_Pin | VCO_LE_Pin;
    GPIO_InitStruct.Mode = GPIO_MODE_OUTPUT_PP;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    GPIO_InitStruct.Speed = GPIO_SPEED_FREQ_LOW;
    HAL_GPIO_Init(GPIOE, &GPIO_InitStruct);

    /*Configure GPIO pin : FAN1_PWN_Pin */
    GPIO_InitStruct.Pin = FAN1_PWN_Pin;
    GPIO_InitStruct.Mode = GPIO_MODE_OUTPUT_PP;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    GPIO_InitStruct.Speed = GPIO_SPEED_FREQ_LOW;
    HAL_GPIO_Init(FAN1_PWN_GPIO_Port, &GPIO_InitStruct);

    /*Configure GPIO pins : LPA_EN_Pin VGA_ATTSEL0_Pin VGA_EN_Pin
       VGA_ATTSEL1_Pin P12V_HSD_DIAG_EN_Pin FAN1_PWR_EN_Pin FAN2_PWR_EN_Pin
       P6V_HDS_ONE_DIAG_EN_Pin */
    GPIO_InitStruct.Pin = LPA_EN_Pin | VGA_ATTSEL0_Pin | VGA_EN_Pin |
                          VGA_ATTSEL1_Pin | P12V_HSD_DIAG_EN_Pin |
                          FAN1_PWR_EN_Pin | FAN2_PWR_EN_Pin |
                          P6V_HDS_ONE_DIAG_EN_Pin;
    GPIO_InitStruct.Mode = GPIO_MODE_OUTPUT_PP;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    GPIO_InitStruct.Speed = GPIO_SPEED_FREQ_LOW;
    HAL_GPIO_Init(GPIOD, &GPIO_InitStruct);

    /*Configure GPIO pin : P6V_PG_Pin */
    GPIO_InitStruct.Pin = P6V_PG_Pin;
    GPIO_InitStruct.Mode = GPIO_MODE_INPUT;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    HAL_GPIO_Init(P6V_PG_GPIO_Port, &GPIO_InitStruct);

    /*Configure GPIO pin : USB_nFAULT_Pin */
    GPIO_InitStruct.Pin = USB_nFAULT_Pin;
    GPIO_InitStruct.Mode = GPIO_MODE_INPUT;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    HAL_GPIO_Init(USB_nFAULT_GPIO_Port, &GPIO_InitStruct);

    /*Configure GPIO pins : MUX_ST_Pin nFAULT_FAN1_Pin nFAULT_FAN2_Pin
     * P6V_HSD_ONE_nFAULT_Pin */
    GPIO_InitStruct.Pin =
        MUX_ST_Pin | nFAULT_FAN1_Pin | nFAULT_FAN2_Pin | P6V_HSD_ONE_nFAULT_Pin;
    GPIO_InitStruct.Mode = GPIO_MODE_INPUT;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    HAL_GPIO_Init(GPIOD, &GPIO_InitStruct);

    /*Configure GPIO pins : P6V_HSD_TWO_nFAULT_Pin VCO_MUXOUT_Pin */
    GPIO_InitStruct.Pin = P6V_HSD_TWO_nFAULT_Pin | VCO_MUXOUT_Pin;
    GPIO_InitStruct.Mode = GPIO_MODE_INPUT;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    HAL_GPIO_Init(GPIOB, &GPIO_InitStruct);

    /*Configure GPIO pins : P6V_HSD_TWO_SEL_Pin P6V_HSD_TWO_SEH_Pin */
    GPIO_InitStruct.Pin = P6V_HSD_TWO_SEL_Pin | P6V_HSD_TWO_SEH_Pin;
    GPIO_InitStruct.Mode = GPIO_MODE_OUTPUT_PP;
    GPIO_InitStruct.Pull = GPIO_NOPULL;
    GPIO_InitStruct.Speed = GPIO_SPEED_FREQ_LOW;
    HAL_GPIO_Init(GPIOB, &GPIO_InitStruct);

    /* EXTI interrupt init*/
    HAL_NVIC_SetPriority(EXTI0_1_IRQn, 3, 0);
    HAL_NVIC_EnableIRQ(EXTI0_1_IRQn);
}

/* USER CODE BEGIN 2 */

/* USER CODE END 2 */
