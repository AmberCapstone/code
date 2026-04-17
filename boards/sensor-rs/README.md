# Sensor Board Firmware

## Option Bytes

The embedded bootloader configures several IO for SPI/I2C/USART bootloading. While most IO configurations wouldn't damage the PCB, PC11 (`USART3_RX`) is configured with a pullup resistor which could damage the SPI Flash and FPGA MISO lines. Therefore, the embedded bootloader must not be used.

To avoid entering the bootloader, set the following option bytes:

| Bit | Value |
|:----|:------|
| `BOOT_LOCK` | 0 |
| `nBOOT1` | 0 |
| `nBOOT_SEL` | 1 |
| `nBOOT0` | 1 |


_See Table 6 in RM0503_


