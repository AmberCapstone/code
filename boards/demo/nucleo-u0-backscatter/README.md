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
This means TX idles LOW, start bit is HIGH.

Data bits are NOT inverted because "Advanced Features - Data Inversion" is also enabled.

## Sample Message

Send two bytes `0xC5, 0xF0`.

```
   ----> TIME
          ___   _       ___   _         _______
         |   | | |     |   | | |       |       |
TX ______|   |_| |_____|   |_| |_______|       |________
   -IDLE--S-1-0-1-0-0-0-1-1-E-S-0-0-0-0-1-1-1-1-E--IDLE-
           |---5---|---C---|   |---0---|---F---|
         |-------BYTE 0------|-------BYTE 1------|
S = Start Bit
E = Stop Bit
```

Sideband is generated when TX=HIGH.

# Settings

## Switching Frequency

6 MHz. Configured under CubeMX - Clock Configuration in bottom-left of the screen.

## Serial

- Baud = 9600 bit/s
- Parity = None
- Stop Bits = 1
- LSB First

