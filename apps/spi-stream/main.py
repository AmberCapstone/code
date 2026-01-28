from __future__ import annotations

import serial
from cobs import Decoder

PORT = "COM3"
BAUD = 2_000_000

def main():
    arduino = serial.Serial(PORT, baudrate=BAUD, timeout=5)
    dec = Decoder()

    print(f"Listening on {PORT} @ {BAUD} baud (COBS)...")\
    
    try:
        while True:
            chunk = arduino.read(256)
            if not chunk:
                continue
            done = dec.Decode(chunk)
            if done:
                print(dec.output.hex(" "))
                dec.output.clear()
                dec = Decoder()

    except KeyboardInterrupt:
        pass
    finally:
        arduino.close()

    
if __name__ == "__main__":
    main()
