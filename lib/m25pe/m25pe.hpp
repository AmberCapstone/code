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
namespace amber::m25pe {

// ==== Memory Layout ====
constexpr uint32_t PAGES_PER_SUBSECTOR = 16;
constexpr uint32_t SUBSECTORS_PER_SECTOR = 16;
constexpr uint32_t NUM_SECTORS = 2;

// All sizes in bytes
constexpr uint32_t PAGE_SIZE = 0x100;
constexpr uint32_t SUBSECTOR_SIZE = PAGE_SIZE * PAGES_PER_SUBSECTOR;
constexpr uint32_t SECTOR_SIZE = SUBSECTOR_SIZE * SUBSECTORS_PER_SECTOR;
constexpr uint32_t TOTAL_SIZE = SECTOR_SIZE * NUM_SECTORS;

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
void PageErase(SpiMaster& spi, uint32_t address);

// Set all bits in `address`'s subsector to 1
void SubsectorErase(SpiMaster& spi, uint32_t address);

// Set all bits in `address`'s sector to 1
void SectorErase(SpiMaster& spi, uint32_t address);

// Set all bits to 1
void BulkErase(SpiMaster& spi);

// Three bytes
// `u32 & 0xff` = Memory capacity
// `(u32 >> 8) & 0xff` = Memory Type
// `(u32 >> 16) & 0xff` = Manufacturer
uint32_t ReadIdentification(SpiMaster& spi);

}  // namespace amber::m25pe
