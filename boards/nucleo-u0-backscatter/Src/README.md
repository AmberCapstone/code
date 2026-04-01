# Wiring

Requires a 74HC00 NAND gate.

| NUCLEO          | 74HC00   |
| :-------------- | :------- |
| 3V3             | VCC (14) |
| GND             | GND (7)  |
| TX (PB6 / D10)  | 1A (1)   |
| MCO (PA10 / D2) | 1B (2)   |

Then connect the "OUTPUT" 74HC00 1Y (Pin 3) to the rf switch control.

# Theory of Operation

Because of the NAND gate:

- When TX is low, 1Y = HIGH (independent of MCO), so the RF switch absorbs energy.
- When TX is high, 1Y switches rapidly due to MCO, so the RF switch generates a sideband.

UART TX pin is INVERTED with "TX Pin Active Level Inversion" under "CubeMX - USART1 - Advanced Features."
This means TX idles LOW.
We still transmit 0 = absorb, 1 = sideband by also enabling "Data Inversion" in CubeMX.

You will see a 1 start bit before the each message starts.

MCO frequency is 6 MHz, configured under CubeMX - Clock Configuration in bottom-left of the screen.
