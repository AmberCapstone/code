import binascii
import subprocess
import time

from tqdm import tqdm
import cobs
import serial
import serial.tools.list_ports
import threading
import sys

if subprocess.call(["sh", "generate_proto.sh"]) != 0:
    exit(1)

import flash_pb2 as flash
from sensor_pb2 import Action, Command, State, Status

PAGE_SIZE = 256
NUM_PAGES = 16 * 16 * 2
TOTAL_SIZE = PAGE_SIZE * NUM_PAGES

PAD_BYTE = b"\0"

ARQ_TIMEOUT = 1

status = Status()


def pack(cmd: Command) -> bytearray:
    return cobs.Encode(cmd.SerializeToString())


def unpack(b: bytes) -> Status | None:
    if not b:
        return None

    decoder = cobs.Decoder()
    if not decoder.Decode(b):
        return None

    s = Status()
    s.ParseFromString(decoder.output)
    return s


def load_binary(file: str) -> bytes | None:
    with open(file, "rb") as f:
        bin = f.read()

    size = len(bin)

    print(f"Loaded {file} ({size} bytes)")
    if size > TOTAL_SIZE:
        print(f"Error: File is too large. Max size is {TOTAL_SIZE} bytes.")
        return None

    elif size < TOTAL_SIZE:
        short = TOTAL_SIZE - size
        print(f"Padding with {short} bytes of {PAD_BYTE} ({TOTAL_SIZE} total bytes).")
        bin = bin + PAD_BYTE * short

    return bin


def connect() -> serial.Serial | None:
    amber_ports = [
        p
        for p in serial.tools.list_ports.comports()
        if p.manufacturer == "amber" and p.product == "Sensor Board"
    ]

    match len(amber_ports):
        case 0:
            print("No amber Sensor Board connected.")
            return None
        case 1:
            return serial.Serial(amber_ports[0].device, timeout=1)
        case _:
            print("Where'd you get two amber Sensor Boards??")
            return None


def reader_thread(ser: serial.Serial, stop: threading.Event):
    global status

    while not stop.is_set():
        sts = unpack(ser.read_until(b"\0"))
        if sts is not None:
            status = sts


if __name__ == "__main__":
    ser = connect()
    if ser is None:
        exit(1)

    bin = load_binary(sys.argv[1])
    if bin is None:
        exit(1)

    stop_signal = threading.Event()

    rt = threading.Thread(target=reader_thread, args=(ser, stop_signal))
    rt.start()

    print("Resetting...")
    while status.state != State.STATE_IDLE:
        ser.write(pack(Command(action=Action.ACTION_RESET)))
        time.sleep(0.1)

    while status.state != State.STATE_FLASHING:
        ser.write(pack(Command(action=Action.ACTION_FLASH)))
        time.sleep(0.1)

    last_tx_time = time.time()

    pbar = tqdm(total=TOTAL_SIZE, unit="byte", desc="Flashing")

    sequence_number = -1
    request_number = -1

    try:
        while not stop_signal.is_set():
            request_number = status.flash_status.page_request

            if status.flash_status.state == flash.State.STATE_DONE:
                stop_signal.set()

            timeout = (time.time() - last_tx_time) > ARQ_TIMEOUT
            if sequence_number < request_number or timeout:
                if not timeout:
                    sequence_number += 1
                    pbar.update(PAGE_SIZE)

                idx = sequence_number * PAGE_SIZE
                d = bin[idx : idx + PAGE_SIZE]
                command = Command(
                    page=flash.Page(
                        page_number=sequence_number,
                        data=d,
                        crc=binascii.crc32(sequence_number.to_bytes(4, "little") + d),
                    )
                )
                ser.write(pack(command))
                last_tx_time = time.time()

            else:
                time.sleep(0.001)

    except KeyboardInterrupt:
        pbar.refresh()
        print("Aborted")

    else:
        pbar.refresh()
        print("Done!")

    finally:
        stop_signal.set()
        rt.join()
