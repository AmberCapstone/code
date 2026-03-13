
# Pinout

| Arduino Name  | Arduino PIN |  FPGA Signal  | FPGA Port | FPGA PIN | Camera Pin |
| :-----------: | :---------: | :-----------: | :-------: | :------: | :--------: |
|      SCK      |     D13     |    SPI_SCK    |           |    15    |            |
|     MISO      |     D12     |   SPI_MISO    |           |    14    |            |
|     MOSI      |     D11     |   SPI_MOSI    |           |    17    |            |
|     SS_n      |     D10     |    SPI_SS     |           |    16    |            |
|      SCL      |     A5      |       -       |     -     |    -     |    SCL     |
|      SDA      |     A4      |       -       |     -     |    -     |    SDA     |
|    capture    |     D2      |    capture    |  IOB_13B  |    6     |     -      |
| capture_ready |     D5      | capture_ready |  IOB_24A  |    13    |     -      |
|       -       |      -      |      d7       | IOB_3b_G6 |    44    |     D7     |
|       -       |      -      |      d6       |  IOB_5B   |    45    |     D6     |
|       -       |      -      |      d5       |  IOB_0A   |    46    |     D5     |
|       -       |      -      |      d4       |  IOB_2A   |    47    |     D4     |
|       -       |      -      |      d3       |  IOB_4A   |    48    |     D3     |
|       -       |      -      |      d2       |  IOB_6A   |    2     |     D2     |
|       -       |      -      |      d1       |  IOB_9B   |    3     |     D1     |
|       -       |      -      |      d0       |  IOB_8A   |    4     |     D0     |
|       -       |      -      |               |           |          |            |

## Notes

-------
`SCK`, `MOSI`, and `SS_n` (D13, D11, D10) are 5V outputs from the Arduino
and need to be stepped down to 3.3 with a 5.1k + 10k resistor divider. `MISO` is 3v3 output from FPGA and can drive the 5V Arduino GPIO.

`capture` (blue wire) goes from Arguino (5v) -> FPGA (3v3), so needs to be stepped down with a 5.1k + 10k resistor divider.

`capture_ready` goes (green wire) goes from FPGA (3v3) to Arduino (5v), and we have tested that this works with no step-up with SPI.

