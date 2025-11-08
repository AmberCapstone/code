from __future__ import annotations

from dataclasses import dataclass

import cobs
import cv2
import numpy as np
import serial


@dataclass
class Packet:
    row: int
    pixels: np.ndarray

    @staticmethod
    def FromBytes(binary: bytes) -> Packet | None:
        if len(binary) != (2 + 640):
            return None

        row = int(binary[0]) | int(binary[1] << 8)
        pixels = np.frombuffer(binary[2:], dtype=np.uint8)

        return Packet(row, pixels)


def main():
    arduino = serial.Serial("/dev/ttyUSB0", baudrate=1000000, timeout=1)

    image = np.zeros((480, 640), dtype=np.uint8)
    decoder = cobs.Decoder()

    for row in range(image.shape[0]):
        image[row, :] = row % 255

    while True:
        if decoder.Decode(arduino.read_until(b"\0")):
            print("received data")
            packet = Packet.FromBytes(decoder.output)
            if packet is None:
                print("Bad packet")
            else:
                image[packet.row] = packet.pixels
            decoder = cobs.Decoder()

        cv2.imshow("Stream", image)
        if cv2.waitKey(1) & 0xFF == ord("q"):
            break

    cv2.destroyAllWindows()


if __name__ == "__main__":
    main()
