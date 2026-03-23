import cv2  # pip install opencv-python

img = cv2.imread("image.png")
if img is None:
    raise ValueError("Failed to load image")
img = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)

print(f"{img.shape=}")

for r in range(img.shape[0]):
    print(f"Row {r:3}: ", end="")
    for i in img[r][0:20]:
        print(f"{i:02x}", end=" ")
    print("")
