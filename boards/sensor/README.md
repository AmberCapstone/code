# STM32U0 SPI Flash with M25PE

## Setup

```bash
git submodule update --init --recursive
```

Open `spi-flash.ioc` in CubeMX and click "Generate Code" to generate the `Drivers/` folder, then run `. post_cubemx.sh`.

## Usage

Break out a USB wire and connect to a Nucleo-U083RC

| USB  | STM32 |
| :--- | :---- |
| D-   | PA11  |
| D+   | PA12  |
| GND  | GND   |
| VBUS | N.C.  |

Connect the M25PE SPI flash to the Nucleo.

| SPI    | STM32   | Description   |
| :----- | :------ | :------------ |
| 1 nS   | PB4 PU  | Chip Select   |
| 2 Q    | PC11    | MISO          |
| 3 nW   | 3.3V PU | Write Protect |
| 4 VSS  | GND     | Ground        |
| 5 D    | PC12    | MOSI          |
| 6 C    | PC10    | SCK           |
| 7 nRST | PB5 PU  | Reset         |
| 8 VCC  | 3.3V    | Supply Volt   |

Connect the energy harvester circuit.

| Harvester | STM32 | Description                        |
| :-------- | :---- | :--------------------------------- |
| GND       | GND   | Ground                             |
| VBAT      | PA4   | Battery ADC (via 2x 1MOhm divider) |

Flash the code to the Nucleo `pio run -t upload`.

Run `app/main.py` to connect to the device.