class Decoder:
    def __init__(self):
        self.output = bytearray()
        self._block_remaining = 0
        self._zero_offset = 0xFF

    @property
    def length(self) -> int:
        return len(self.output)

    def Decode(self, encoded: bytes) -> bool:
        for b in encoded:
            if self._block_remaining > 0:
                self.output.append(b)
            else:
                self._block_remaining = b
                if self._block_remaining == 0:
                    return True

                if self._zero_offset != 0xFF:
                    self.output.append(0x00)
                self._zero_offset = self._block_remaining

            self._block_remaining -= 1

        return False


def Encode(raw: bytes) -> bytearray:
    TEMP_OFFSET = 0x00  # will be replaced once offset is known

    output = bytearray([TEMP_OFFSET])
    zero_offset = 1
    zero_offset_idx = 0

    for b in raw:
        if b != 0x00:
            output.append(b)
            zero_offset += 1

        if (b == 0x00) or (zero_offset == 0xFF):
            output[zero_offset_idx] = zero_offset
            output.append(TEMP_OFFSET)
            zero_offset_idx = len(output) - 1
            zero_offset = 1

    output[zero_offset_idx] = zero_offset
    output.append(0x00)

    return output
