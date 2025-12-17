#include "flash.hpp"

#include "spi_master.hpp"

// CubeMX
#include "gpio.h"
#include "spi.h"

namespace flash {

class SPI : public amber::SpiMaster {
public:
    SPI(SPI_HandleTypeDef* hspi, GPIO_TypeDef* cs_n_port, uint16_t cs_n_pin)
        : hspi_(hspi), cs_n_port_(cs_n_port), cs_n_pin_(cs_n_pin) {}

    void Transmit(uint8_t* tx_data, uint32_t length) override {
        HAL_SPI_Transmit(hspi_, tx_data, length, 100);
    }

    void Receive(uint8_t* rx_data, uint32_t length) override {
        HAL_SPI_Receive(hspi_, rx_data, length, 100);
    }

    void TransmitReceive(uint8_t* tx_data, uint8_t* rx_data,
                         uint32_t length) override {
        HAL_SPI_TransmitReceive(hspi_, tx_data, rx_data, length, 100);
    }

    void SetChipSelect(bool high) override {
        HAL_GPIO_WritePin(cs_n_port_, cs_n_pin_,
                          static_cast<GPIO_PinState>(high));
    }

private:
    SPI_HandleTypeDef* hspi_;
    GPIO_TypeDef* cs_n_port_;
    uint16_t cs_n_pin_;
};

static SPI m25_spi(&hspi1, M25_nCHIP_SELECT_GPIO_Port, M25_nCHIP_SELECT_Pin);

void Init(void) {
    HAL_GPIO_WritePin(M25_nWRITE_PROTECT_GPIO_Port, M25_nWRITE_PROTECT_Pin,
                      GPIO_PIN_SET);
    HAL_GPIO_WritePin(M25_nRESET_GPIO_Port, M25_nRESET_Pin, GPIO_PIN_SET);
    HAL_GPIO_WritePin(M25_nCHIP_SELECT_GPIO_Port, M25_nCHIP_SELECT_Pin,
                      GPIO_PIN_SET);
}

}  // namespace flash