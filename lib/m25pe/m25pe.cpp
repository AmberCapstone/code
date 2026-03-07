#include "m25pe.hpp"

#if __has_include(<cstdint>)
#include <cstdint>
#else
#include <stdint.h>
#endif

#include "spi_master.hpp"

namespace amber::m25pe {

inline void SendCommand(SpiMaster& spi, Command command) {
    uint8_t tx = static_cast<uint8_t>(command);
    spi.Transmit(&tx, 1);
}

static inline void SendAddress(SpiMaster& spi, uint32_t address) {
    uint8_t tx[3] = {
        static_cast<uint8_t>((address >> 16) & 0xff),
        static_cast<uint8_t>((address >> 8) & 0xff),
        static_cast<uint8_t>(address & 0xff),
    };
    spi.Transmit(tx, 3);
}

bool IsWriteInProgress(SpiMaster& spi) {
    uint8_t status;

    spi.SetChipSelect(false);
    SendCommand(spi, Command::READ_STATUS_REGISTER);
    spi.Receive(&status, 1);
    spi.SetChipSelect(true);

    return status & (1 << StatusRegister::WIP);
}

void EnableWriting(SpiMaster& spi) {
    spi.SetChipSelect(false);
    SendCommand(spi, Command::WRITE_ENABLE);
    spi.SetChipSelect(true);
}

void DisableWriting(SpiMaster& spi) {
    spi.SetChipSelect(false);
    SendCommand(spi, Command::WRITE_DISABLE);
    spi.SetChipSelect(true);
}

void PowerDown(SpiMaster& spi) {
    spi.SetChipSelect(false);
    SendCommand(spi, Command::DEEP_POWER_DOWN);
    spi.SetChipSelect(true);
}

void WakeUp(SpiMaster& spi) {
    spi.SetChipSelect(false);
    SendCommand(spi, Command::RELEASE_DEEP_POWER_DOWN);
    spi.SetChipSelect(true);
}

void PageProgram(SpiMaster& spi, uint32_t address, uint8_t* data,
                 uint32_t length) {
    spi.SetChipSelect(false);
    SendCommand(spi, Command::PAGE_PROGRAM);
    SendAddress(spi, address);
    spi.Transmit(data, length);
    spi.SetChipSelect(true);
}

void PageWrite(SpiMaster& spi, uint32_t address, uint8_t* data,
               uint32_t length) {
    spi.SetChipSelect(false);
    SendCommand(spi, Command::PAGE_WRITE);
    SendAddress(spi, address);
    spi.Transmit(data, length);
    spi.SetChipSelect(true);
}

void ReadData(SpiMaster& spi, uint32_t address, uint32_t length, uint8_t* out) {
    spi.SetChipSelect(false);
    SendCommand(spi, Command::READ_DATA_BYTES);
    SendAddress(spi, address);
    spi.Receive(out, length);
    spi.SetChipSelect(true);
}

void PageErase(SpiMaster& spi, uint32_t address) {
    spi.SetChipSelect(false);
    SendCommand(spi, Command::PAGE_ERASE);
    SendAddress(spi, address);
    spi.SetChipSelect(true);
}

void SubsectorErase(SpiMaster& spi, uint32_t address) {
    spi.SetChipSelect(false);
    SendCommand(spi, Command::SUBSECTOR_ERASE);
    SendAddress(spi, address);
    spi.SetChipSelect(true);
}

void SectorErase(SpiMaster& spi, uint32_t address) {
    spi.SetChipSelect(false);
    SendCommand(spi, Command::SECTOR_ERASE);
    SendAddress(spi, address);
    spi.SetChipSelect(true);
}

void BulkErase(SpiMaster& spi) {
    spi.SetChipSelect(false);
    SendCommand(spi, Command::BULK_ERASE);
    spi.SetChipSelect(true);
}

uint32_t ReadIdentification(SpiMaster& spi) {
    uint8_t bytes[3];

    spi.SetChipSelect(false);
    SendCommand(spi, Command::READ_IDENTIFICATION);
    spi.Receive(bytes, 3);
    spi.SetChipSelect(true);

    uint32_t out = bytes[0] << 16 | bytes[1] << 8 | bytes[2];
    return out;
}
}  // namespace amber::m25pe
