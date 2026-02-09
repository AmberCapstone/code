from __future__ import annotations

import time
import serial
from cobs import Decoder, Encode

PORT = "COM3"
BAUD = 2_000_000

OPC_NOP = 0x00
OPC_WR = 0x01
OPC_RD = 0x02

READ_LATENCY = 4

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

def cmd_init() -> bytes:
    return bytes ([OPC_INIT, 0, 0, 0, 0, 0, 0, 0x11])

def main():
    with serial.Serial(PORT, baudrate=BAUD, timeout=0.5) as ser:
        print(f"Connected to {PORT} @ {BAUD} baud")

        time.sleep(2.0)

        ser.reset_input_buffer()
        ser.reset_output_buffer()

        # send_frame(ser, cmd_init())
        # resp = read_frame(ser, timeout_s=1.0)
        # print("INIT resp:", resp.hex(" ") if resp else "<timeout>")

        time.sleep(0.5)


        addr = 0x00
        # data = bytes([0xAA, 0xBB, 0xCC, 0xDD, 0xAA, 0xBB, 0xCC, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77])  # example
        data = bytes([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF])  # example
        payload = bytes([OPC_WR, addr]) + data
        send_frame(ser, payload)
        # chunk = ser.read(256)
        # print("chunk:", chunk.hex(" "))
        resp = read_frame(ser)   # will just be same length echo from Arduino
        # print("response:", resp.hex(" "))
        # addr = 0x07
        # data = bytes([0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77])  # example
        # payload = bytes([OPC_WR, addr]) + data
        # send_frame(ser, payload)
        # # chunk = ser.read(256)
        # # print("chunk:", chunk.hex(" "))
        # resp = read_frame(ser)   # will just be same length echo from Arduino
        # print("response:", resp.hex(" "))

        addr = 0x00
        payload = bytes([OPC_RD, addr]) + bytes([0x00]*7)
        send_frame(ser, payload)
        resp = read_frame(ser)
        # resp is 18 bytes; returned data will be in resp[2:18]
        print("response:\t", resp.hex(" "))

        # data = bytes.fromhex("AA AA AA AA AA AA AA AA AA AA")
        # send_frame(ser, bytes([OPC_WR]) + data)
        # resp = read_frame(ser, timeout_s=1.0)
        # print("WR resp:", resp.hex(" ") if resp else "<timeout>")

        # send_frame(ser, bytes([OPC_RD]) + data)
        # resp = read_frame(ser)
        # print("Receivd:", resp.hex(" "))

if __name__ == "__main__":
    main()
