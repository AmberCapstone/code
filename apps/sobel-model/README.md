# Sobel Model
Calculate Sobel Gradients and sum together to find sharpness measure of an image. 

## Install

```
pip install -r requirements.txt
```

## Usage

Takes images from blurgen.py
Pass the resolution then `-b/--blur` for blur amount. Uses Gaussian blur with the specified sigma (in pixels).

```bash
python blurgen.py QQVGA -b 8
```

![](doc/qqvga.png)

## Checker Size

Change the square size (in pixels) with `-s/--size`.

```bash
python blurgen.py QQVGA -b 8 -s 40
```

![](doc/size.png)

## Resolutions

Enter `width,height` for specific dimensions, or use a preset like VGA or QQVGA. Pass `--help` to see all presets.

```bash
python blurgen.py 120,240 -b 4
```

![](doc/tall.png)

## Generate a Series of Images

Pass multiple blur values seperated by spaces.

```bash
$ python blurgen.py QQVGA -b 0 4 8 12
Saved out_b0.png
Saved out_b4.png
Saved out_b8.png
Saved out_b12.png
```

![](doc/sequence_b0.png)
![](doc/sequence_b4.png)
![](doc/sequence_b8.png)
![](doc/sequence_b12.png)

## Output Format

File format defaults to PNG. Change it with `-f/--fmt` to one of PGM, PNG, PPM, or YUV422.

```bash
$ python blurgen.py 8,8 -s2 --fmt YUV422
Saved out.yuv422
```

```text
 Y  U  Y  V   Y  U  Y  V   Y  U  Y  V   Y  U  Y  V
00 00 00 00  FF 00 FF 00  00 00 00 00  FF 00 FF 00
00 00 00 00  FF 00 FF 00  00 00 00 00  FF 00 FF 00

FF 00 FF 00  00 00 00 00  FF 00 FF 00  00 00 00 00
FF 00 FF 00  00 00 00 00  FF 00 FF 00  00 00 00 00

00 00 00 00  FF 00 FF 00  00 00 00 00  FF 00 FF 00
00 00 00 00  FF 00 FF 00  00 00 00 00  FF 00 FF 00

FF 00 FF 00  00 00 00 00  FF 00 FF 00  00 00 00 00
FF 00 FF 00  00 00 00 00  FF 00 FF 00  00 00 00 00
```

> YUV422 saves a binary file with four bytes `[Y1 U12 Y2 V12]` per two pixels in row-major order. The sample above is a textual visualization, not the raw output.

## Output Filename

Default is `out`. Change it with `-o/--output`.

```bash
$ python blurgen.py VGA -b 0 10 20 --fmt pgm -o vga_dataset
Saved vga_dataset_b0.pgm
Saved vga_dataset_b10.pgm
Saved vga_dataset_b20.pgm
```
