#include <Arduino.h>
#include <SPI.h>

#include "m25pe.hpp"
#include "spi_master.hpp"

using namespace amber;

SPISettings settings{4000000, MSBFIRST, SPI_MODE0};

class ArduinoSpi : public SpiMaster {
public:
    ArduinoSpi(SPIClass spi, uint8_t cs_pin) : spi_(spi), cs_pin_(cs_pin) {
        spi_.begin();
    }

    void SetChipSelect(bool high) override {
        digitalWrite(cs_pin_, high);
    }

    void Transmit(uint8_t* tx_data, uint32_t length) override {
        spi_.beginTransaction(settings);
        spi_.transfer(tx_data, length);
        spi_.endTransaction();
    }
    void Receive(uint8_t* rx_data, uint32_t length) override {
        spi_.beginTransaction(settings);
        spi_.transfer(rx_data, length);
        spi_.endTransaction();
    }

    // will not work, don't need for this demo
    void TransmitReceive(uint8_t* tx_data, uint8_t* rx_data,
                         uint32_t length) override {
        spi_.beginTransaction(settings);
        spi_.transfer(tx_data, length);
        spi_.endTransaction();
    }

private:
    SPIClass spi_;
    uint8_t cs_pin_;
};

const uint8_t M25_SCK = 13;
const uint8_t M25_MISO = 12;
const uint8_t M25_MOSI = 11;

const uint8_t M25_RESETn = 10;
const uint8_t M25_WRITE_PROTECTn = 9;
const uint8_t M25_CSn = 8;

ArduinoSpi ard_spi{SPI, M25_CSn};

uint8_t rx_buffer[256];
uint32_t rx_buffer_len;
uint32_t address;

void setup() {
    pinMode(M25_CSn, OUTPUT);
    pinMode(M25_WRITE_PROTECTn, OUTPUT);
    pinMode(M25_RESETn, OUTPUT);

    digitalWrite(M25_WRITE_PROTECTn, HIGH);
    digitalWrite(M25_RESETn, HIGH);
    digitalWrite(M25_CSn, HIGH);

    Serial.begin(115200);

    SPI.begin();
}

enum State {
    IDLE,
    RECEIVING,
    WRITING,
    READING,
    ERASING,
};

State state = IDLE;
bool on_enter = true;

void loop() {
    State new_state = state;
    switch (state) {
        case IDLE:
            if (on_enter) {
                Serial.println(
                    "Ready to interact. Send the following in HEX. Address is "
                    "3 bytes, LSB sent first.");
                Serial.println("Write  x57|address|byte0,byte1,...byteN");
                Serial.println("Read   x52|address");
                Serial.println("Erase  x45|address");
            }
            if (Serial.available()) {
                new_state = RECEIVING;
            }
            break;

        case RECEIVING: {
            if (on_enter) {
                Serial.println("Receiving...");
            }
            char command = Serial.read();

            uint8_t addr_bytes[3] = {0};
            Serial.readBytes(addr_bytes, 3);

            address = addr_bytes[0] |
                      static_cast<uint32_t>(addr_bytes[1]) << 8 |
                      static_cast<uint32_t>(addr_bytes[2]) << 16;

            switch (command) {
                case 'W':  // 0x57
                    rx_buffer_len = Serial.readBytes(rx_buffer, 256);
                    new_state = WRITING;
                    break;
                case 'R':  // 0x52
                    new_state = READING;
                    break;
                case 'E':  // 0x45
                    new_state = ERASING;
                    break;

                default:
                    Serial.println("Invalid command");
                    new_state = IDLE;
            }

            break;
        }

        case WRITING: {
            if (on_enter) {
                Serial.print("Writing ");
                Serial.print(rx_buffer_len);
                Serial.print(" bytes to ");
                Serial.println(address, HEX);

                m25pe::EnableWriting(ard_spi);
                m25pe::PageWrite(ard_spi, address, rx_buffer, rx_buffer_len);
            }

            if (m25pe::IsWriteInProgress(ard_spi)) {
                Serial.println("writing...");
            } else {
                new_state = IDLE;
            }

            break;
        }

        case READING:
            m25pe::ReadData(ard_spi, address, 256, rx_buffer);

            for (int8_t row = 15; row >= 0; row--) {
                for (int8_t col = 15; col >= 0; col--) {
                    Serial.print(rx_buffer[row * 16 + col], HEX);
                    Serial.print("\t");
                }
                Serial.println("");
            }

            new_state = IDLE;
            break;

        case ERASING:
            if (on_enter) {
                m25pe::EnableWriting(ard_spi);
                m25pe::PageErase(ard_spi, address);
            }

            if (m25pe::IsWriteInProgress(ard_spi)) {
                Serial.println("erasing...");
            } else {
                new_state = IDLE;
            }

            break;
    }
    on_enter = new_state != state;
    if (on_enter) {
        state = new_state;
    }
}