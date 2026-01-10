# STM32U0 USB Demo

## Setup

```bash
git submodule update --init --recursive
```

Open `u0-usb-demo.ioc` in CubeMX and click "Generate Code" to generate the `Drivers/` folder, then run `. post_cubemx.sh`.

## Usage

Break out a USB wire and connect to a Nucleo-U083RC

| USB  | STM32 |
|:-----|:------|
| D-   | PA11  |
| D+   | PA12  |
| GND  | GND   |
| VBUS | N.C.  |

Flash the code to the Nucleo `pio run -t upload`.

Open a Serial Monitor. You should see an `amber` device. Connect to it and start monitoring. Baud rate doesn't matter for USB serial.

```text
---- Opened the serial port /dev/ttyACM1 - amber ----
The counter is 8
The counter is 9
The counter is 10
The counter is 11
The counter is 12
```
