import gzip
import numpy as np
import matplotlib.pyplot as plt

# Load the file (unzip it first if necessary)
filename = 'train-images-idx3-ubyte'
with open(filename, 'rb') as f:
    # Skip the header (16 bytes)
    f.read(16)
    # Read the rest of the bytes
    buf = f.read()
    # Convert bytes to numpy array
    data = np.frombuffer(buf, dtype=np.uint8)
    # Reshape to (number_of_images, 28, 28)
    data = data.reshape(-1, 28, 28)

# View the first image
plt.imshow(data[1], cmap='gray')
plt.show()
