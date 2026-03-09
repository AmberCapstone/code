import binascii
import subprocess
import sys
import threading
import time

import cobs
import serial
import serial.tools.list_ports
from tqdm import tqdm

if subprocess.call(["sh", "generate_proto.sh"]) != 0:
    exit(1)

import sensor.flash_pb2 as flash
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

    filename = sys.argv[1]
    bin = load_binary(filename)
    if bin is None:
        exit(1)

    stop_signal = threading.Event()

    rt = threading.Thread(target=reader_thread, args=(ser, stop_signal))
    rt.start()

    print("Resetting...")
    while status.state != State.STATE_IDLE:
        ser.write(pack(Command(action=Action.ACTION_RESET)))
        time.sleep(0.1)

    while (
        status.state != State.STATE_FLASHING
        and status.flash_status.state != flash.State.STATE_ERASING
    ):
        ser.write(pack(Command(action=Action.ACTION_FLASH)))
        time.sleep(0.1)

    print("Erasing...")
    while status.flash_status.state == flash.State.STATE_ERASING:
        time.sleep(0.1)

    pbar = tqdm(total=TOTAL_SIZE, unit="byte", desc="Programming", ascii=" >=")

    last_tx_time = time.time()
    sequence_number = 0
    request_number = -1

    expected_crcs = [0] * NUM_PAGES

    while status.flash_status.state == flash.State.STATE_PROGRAMMING:
        request_number = status.flash_status.stm_page_request

        timeout = (time.time() - last_tx_time) > ARQ_TIMEOUT
        if request_number > sequence_number or timeout:
            if request_number > sequence_number:
                sequence_number = request_number
                pbar.update(PAGE_SIZE)

            idx = sequence_number * PAGE_SIZE
            db: bytes = bin[idx : idx + PAGE_SIZE]
            crc = binascii.crc32(sequence_number.to_bytes(4, "little") + db)
            expected_crcs[sequence_number] = crc
            command = Command(
                page=flash.Page(
                    page_number=sequence_number,
                    data=db,
                    crc=crc,
                )
            )
            ser.write(pack(command))
            last_tx_time = time.time()

        else:
            time.sleep(0.002)

    pbar.refresh()
    pbar.close()

    while status.state != State.STATE_READOUT:
        ser.write(pack(Command(action=Action.ACTION_READOUT)))
        time.sleep(0.1)

    pbar = tqdm(total=NUM_PAGES, unit="page", desc="Verifying", ascii=" >=")

    request_number = 0
    errors = []
    full_data = bytearray()
    while status.state == State.STATE_READOUT:
        command = Command(host_page_request=request_number)
        ser.write(pack(command))

        if not status.flash_status.HasField("readout_page"):
            continue

        page = status.flash_status.readout_page

        if page.page_number >= NUM_PAGES:
            continue

        if page.page_number == request_number:
            request_number += 1
            pbar.update(1)
            full_data.extend(page.data)
            if expected_crcs[page.page_number] != page.crc:
                errors.append(
                    {
                        "page": page.page_number,
                        "expected_crc": expected_crcs[page.page_number],
                        "actual_crc": page.crc,
                        "data": page,
                    }
                )

        time.sleep(0.002)

    pbar.refresh()
    pbar.close()

    print(f"{len(errors)} error(s)")
    if len(errors) != 0:
        print("Page  | Exp. CRC | Act. CRC")
        for e in errors:
            print(
                f"0x{e['page']:03X} | {e['expected_crc']:08x} | {e['actual_crc']:08x}"
            )
            d: flash.Page = e["data"]
            print(f"pagenum: {d.page_number}")
            for i in range(256):
                if d.data[i] == i:
                    print(f" {d.data[i]:02x} ", end="")
                else:
                    print(f"[{d.data[i]:02x}]", end="")
                if i % 16 == 15:
                    print("")
            print(f"crc = {d.crc} = 0x{d.crc:08x}")
            print("------------")

    print("Done!")

    stop_signal.set()
    rt.join()

    readout_filename = filename + ".ro"
    with open(readout_filename, "wb") as f:
        f.write(full_data)
        print(f"Readout saved to {readout_filename}")
