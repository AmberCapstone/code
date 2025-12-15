#pragma once

#if __has_include(<cstdint>)
#include <cstdint>
#else
#include <stdint.h>
#endif

#include "spi_master.hpp"

// ======== M25PE ========
// Caller is responsible for checking `IsWriteInProgress()` before starting a
// new write operation.
//
// ==== Memory Layout ====
// 512 Pages (256 bytes each)
// 32 Subsectors (4096 bytes = 16 pages each)
// 2 Sectors (65536 bytes = 16 subsectors each)
namespace amber::m25pe {

enum StatusRegister {
    WIP = 0,
    WEL = 1,
    BP0 = 2,
    BP1 = 3,
    SRWD = 7
};

enum class Command : uint8_t {
    WRITE_ENABLE = 0x06,
    WRITE_DISABLE = 0x04,
    READ_IDENTIFICATION = 0x9f,
    READ_STATUS_REGISTER = 0x05,
    WRITE_STATUS_REGISTER = 0x01,
    WRITE_TO_LOCK_REGISTER = 0xe5,
    READ_LOCK_REGISTER = 0xe8,
    READ_DATA_BYTES = 0x03,
    READ_DATA_BYTES_HIGH_SPEED = 0x0b,
    PAGE_WRITE = 0x0a,
    PAGE_PROGRAM = 0x02,
    PAGE_ERASE = 0xdb,
    SUBSECTOR_ERASE = 0x20,
    SECTOR_ERASE = 0xd8,
    BULK_ERASE = 0xc7,
    DEEP_POWER_DOWN = 0xb9,
    RELEASE_DEEP_POWER_DOWN = 0xab,
};

void SendCommand(SpiMaster& spi, Command command);

// ---------- Control Commands ----------

void EnableWriting(SpiMaster& spi);
void DisableWriting(SpiMaster& spi);
void PowerDown(SpiMaster& spi);
void WakeUp(SpiMaster& spi);
bool IsWriteInProgress(SpiMaster& spi);

// ---------- Write Commands ----------

/// Can only set bits from 1 -> 0.
void PageProgram(SpiMaster& spi, uint32_t address, uint8_t* data,
                 uint32_t length);

/// Can set bits from 0 -> 1 and 1 -> 0. Erases the entire page first.
void PageWrite(SpiMaster& spi, uint32_t address, uint8_t* data,
               uint32_t length);

// ---------- Read Commands ----------

/// Read `length` bytes starting from `address`.
/// Caller is responsible for not overflowing `out`
void ReadData(SpiMaster& spi, uint32_t address, uint32_t length, uint8_t* out);

// ---------- Erase Commands ----------

// Set all bits `address`'s page to 1
// Each page has 0x100 bytes
void PageErase(SpiMaster& spi, uint32_t address);

// Set all bits in `address`'s subsector to 1
// Each subsector has 0x1000 bytes
void SubsectorErase(SpiMaster& spi, uint32_t address);

// Set all bits in `address`'s sector to 1
// Each sector has 0x10000 bytes
void SectorErase(SpiMaster& spi, uint32_t address);

// Set all bits to 1
void BulkErase(SpiMaster& spi);

}  // namespace amber::m25pe
