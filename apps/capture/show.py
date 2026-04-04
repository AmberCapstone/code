import cv2  # pip install opencv-python

import matplotlib.pyplot as plt

img = cv2.imread("image.png")
if img is None:
    raise ValueError("Failed to load image")
img = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)


plt.imshow(img)
plt.show()
