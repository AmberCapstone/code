from __future__ import annotations


import os
import time
import cv2
import numpy as np
import serial

PORT = "COM3"
BAUD = 2_000_000


def main():
    arduino = serial.Serial(PORT, baudrate=BAUD, timeout=5)

    W = 640 + 10  # buffer for stray pixels? unsure why some rows are longer
    H = 480

    image = np.zeros((H, W), dtype=np.uint8)

    for row in range(image.shape[0]):
        image[row, :] = row % 255

    row = 0

    while True:
        row_data = arduino.read_until(b"\0")[:-1]  # remove 0 byte
        pixels = np.frombuffer(row_data, dtype=np.uint8)

        try:
            image[row, : pixels.size] = pixels
            # draw inverted "cursor" line to show where the image is updating
            image[(row + 1) % H, : pixels.size] = 255 - pixels

        except ValueError:
            continue

        row = (row + 1) % H

        cv2.imshow("Stream", image)
        key = cv2.waitKey(1) & 0xFF

        if key == ord("q"):  # quit application
            break

        elif key == ord("r"):  # reset first row
            row = 0

        elif key == ord("s"):  # save image
            os.makedirs("captures/", exist_ok=True)
            timestamp = time.strftime("%Y-%d-%m_%H%M%S")
            file = f"captures/{timestamp}.png"
            cv2.imwrite(file, image)
            print(f"Saved to {file}")

    cv2.destroyAllWindows()


if __name__ == "__main__":
    main()
