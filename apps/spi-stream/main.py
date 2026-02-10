from __future__ import annotations

import time
import serial
import numpy as np
import matplotlib.pyplot as plt
from cobs import Decoder, Encode

PORT = "COM3"
BAUD = 500_000

OPC_NOP = 0x00
OPC_WR = 0x01
OPC_RD = 0x02

MAX_CHUNK = 6
TOTAL_PIXELS = 784


def load_mnist_image(filename: str, index: int) -> bytes:
    with open(filename, "rb") as f:
        f.read(16) # header
        buf = f.read()

    data = np.frombuffer(buf, dtype=np.uint8)
    images = data.reshape(-1, 28, 28)
    
    img = images[index]

    plt.figure("MNIST", clear=True)
    plt.imshow(img, cmap='gray')
    # plt.show()
    # plt.show(block=False)
    # plt.pause(1)
    return img.flatten().tobytes()

def read_frame(ser: serial.Serial, timeout_s: float = 1) -> bytes | None:
    dec = Decoder()
    deadline = time.time() + timeout_s

    while time.time() < deadline:
        chunk = ser.read(256)
        if not chunk:
            continue
        if dec.Decode(chunk):
            payload = bytes(dec.output)
            return payload
        
    return None

def send_frame(ser: serial.Serial, payload: bytes) -> None:
    print("payload:\t", payload.hex(" "))
    encoded = bytes(Encode(payload))
    ser.write(encoded)

def main():
    with serial.Serial(PORT, baudrate=BAUD, timeout=0.5) as ser:
        print("Loading MNIST image")
        mnist_bytes = load_mnist_image("train-images-idx3-ubyte", index=1)
        assert len(mnist_bytes) == 784

        print(f"Connected to {PORT} @ {BAUD} baud")
        time.sleep(2.0)
        ser.reset_input_buffer()
        ser.reset_output_buffer()
        time.sleep(0.5)

        writes = 0
        base_addr = 0
        while writes < TOTAL_PIXELS:
            n = min(MAX_CHUNK, TOTAL_PIXELS - writes)
            addr = base_addr + writes
            addr_hi = (addr >> 8) & 0xFF
            addr_lo = addr & 0xFF
            data = mnist_bytes[writes : writes + n]

            payload = bytes([OPC_WR, addr_hi, addr_lo]) + data
            send_frame(ser, payload)
            resp = read_frame(ser)
            if resp is None:
                print(f"Noresponse on WR @ {addr}")
            writes += n


        reads = 0
        while reads < TOTAL_PIXELS:
            n = min(MAX_CHUNK, TOTAL_PIXELS - reads)
            addr = base_addr + reads
            addr_hi = (addr >> 8) & 0xFF
            addr_lo = addr & 0xFF
            
            expected = mnist_bytes[reads : reads + n]

            payload = bytes([OPC_RD, addr_hi, addr_lo]) + bytes([0x00]*n)
            send_frame(ser, payload)

            resp = read_frame(ser)
            if resp is None:
                print(f"No response on RD @ {addr}")
                reads += n
                continue
                
            got = resp[3 : 3 + n]
            if got == expected:
                print("Pass: response:\t", got.hex(" "), "\texpected:\t", expected.hex(" "))
            else:
                print("ERROR =: response:\t", got.hex(" "), "\texpected:\t", expected.hex(" "))

            reads += n

        
        # for i in range(0, TOTAL_PIXELS, MAX_CHUNK):
        #     addr = i
        #     addr_hi = (addr >> 8) & 0xFF
        #     addr_lo = addr & 0xFF
        #     data = bytes([0xAA & i, 0xCC & i, 0xDD & i, 0xEE & i, 0xFF & i, 0x12])  # example
        #     payload = bytes([OPC_WR, addr_hi, addr_lo]) + data
        #     send_frame(ser, payload)
        #     resp = read_frame(ser)


        

        # for i in range(0, TOTAL_PIXELS, MAX_CHUNK):
        #     addr = i
        #     addr_hi = (addr >> 8) & 0xFF
        #     addr_lo = addr & 0xFF
        #     payload = bytes([OPC_RD, addr_hi, addr_lo]) + bytes([0x00]*6)
        #     data = bytes([0xAA & i, 0xCC & i, 0xDD & i, 0xEE & i, 0xFF & i, 0x12])  # example
        #     send_frame(ser, payload)
        #     resp = read_frame(ser)
        #     if (resp[3:9] == data): 
        #         print("Pass")
        #     else: 
        #         print("Fail: response:\t", resp.hex(" "), "\texpected:\t", data.hex(" "))
        #     print("response:\t", resp.hex(" "), "\texpected:\t", data.hex(" "))




if __name__ == "__main__":
    main()
