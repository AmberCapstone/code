/** 
 * @file spi.hpp
 * @author Ivan Lange
 * @brief SPI driver wrapper
 * 
 * @date 2026-03-21
 */

#pragma once

#include <cstdint>

#include "periph/digital.hpp"

#ifdef STM32F7
#include "stm32f7xx_hal.h"
#elif STM32G0
#include "stm32g0xx_hal.h"

#endif

#ifdef HAL_SPI_MODULE_ENABLED

namespace amber::periph {

struct Spi {

    Spi(SPI_HandleTypeDef& hspi, amber::periph::DigitalOutput& csPin)
        : _hspi(hspi), _csPin(csPin) {}

    ~Spi() = default;

    auto transmit(const uint8_t* data, uint16_t len) noexcept -> HAL_StatusTypeDef {
        csAssert();
        auto status = HAL_SPI_Transmit(&_hspi, const_cast<uint8_t*>(data), len, HAL_MAX_DELAY);
        csRelease();
        return status;
    }

    auto receive(uint8_t* data, uint16_t len) noexcept -> HAL_StatusTypeDef {
        csAssert();
        auto status = HAL_SPI_Receive(&_hspi, data, len, HAL_MAX_DELAY);
        csRelease();
        return status;
    }

    auto transceive(const uint8_t* tx, uint8_t* rx, uint16_t len) noexcept -> HAL_StatusTypeDef {
        csAssert();
        auto status = HAL_SPI_TransmitReceive(&_hspi, const_cast<uint8_t*>(tx), rx, len, HAL_MAX_DELAY);
        csRelease();
        return status;
    }

    auto transmitThenReceive(const uint8_t* tx, uint16_t txLen,
                            uint8_t* rx, uint16_t rxLen) noexcept -> HAL_StatusTypeDef {
        csAssert();
        auto status = HAL_SPI_Transmit(&_hspi, const_cast<uint8_t*>(tx), txLen, HAL_MAX_DELAY);
        if (status == HAL_OK) {
            status = HAL_SPI_Receive(&_hspi, rx, rxLen, HAL_MAX_DELAY);
        }
        csRelease();
        return status;
    }

private:
    void csAssert() {
        _csPin.SetLow();
    }

    void csRelease() {
        _csPin.SetHigh();
    }

    SPI_HandleTypeDef& _hspi;
    amber::periph::DigitalOutput& _csPin;
};

}  // namespace amber::periph

#endif  // HAL_SPI_MODULE_ENABLED
