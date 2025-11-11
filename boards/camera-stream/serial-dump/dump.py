import cobs
import serial

PORT = r"/dev/ttyUSB0"
BAUD = 250_000

ser = serial.Serial(PORT, BAUD)

try:
    while True:
        data = ser.read_until(b"\0")
        decoder = cobs.Decoder()
        decoder.Decode(data)
        print(decoder.output.decode())

except KeyboardInterrupt:
    print("\nClosing...")
    ser.close()
