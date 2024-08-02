#!/usr/bin/env python

import gzip
import os
import struct
import json


def generate_random_data(file_path: str, size: int) -> None:
    with open(file_path, 'wb') as f:
        f.write(os.urandom(size))


def generate_repeating_data(file_path: str, size: int, repeat: int) -> None:
    with open(file_path, 'wb') as f:
        f.write(os.urandom(size) * repeat)


def generate_lorem_ipsum(file_path: str, size: int) -> None:
    lorem = b"""Lorem ipsum dolor sit amet, consectetur adipiscing elit
    sed do eiusmod tempor incididunt ut labore et dolore magna aliqua
    Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris
    nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in
    reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla
    """
    with open(file_path, 'wb') as f:
        f.write(lorem * (size // len(lorem)))


def compress_with_gzip(input_file: str, output_file: str) -> None:
    with open(input_file, 'rb') as f_in:
        with gzip.open(output_file, 'wb') as f_out:
            f_out.writelines(f_in)


DATA_DIR = 'data'

MANIFEST_FILE = 'manifest.json'

GENERATORS = {
    'random_data': [generate_random_data, 512],
    'repeat_1_data': [generate_repeating_data, 8, 64],
    'repeat_2_data': [generate_repeating_data, 16, 31],
    'lorem_ipsum_data': [generate_lorem_ipsum, 1024],
}


def get_gzip_name(file_name: str) -> str:
    return file_name + '.gz'


def get_deflate_name(file_name: str) -> str:
    return file_name + '.deflate'


def get_working_dir() -> str:
    return os.path.dirname(os.path.realpath(__file__))


def get_data_path(file_name: str) -> str:
    return os.path.join(get_working_dir(), DATA_DIR, file_name)


def extract_deflate_data(gzip_file_path: str, deflate_file_path: str) -> None:
    with open(gzip_file_path, 'rb') as f:
        # Skip the 10-byte header
        f.seek(3)

        # Read the extra flags (FLG)
        flg = ord(f.read(1))

        f.seek(6)  # Skip the modification time, extra flags, and OS identifier

        # Skip optional fields based on the flags
        if flg & 0x04:  # If the FEXTRA flag is set
            xlen = struct.unpack('<H', f.read(2))[0]
            f.seek(xlen, 1)  # Skip the extra field
        if flg & 0x08:  # If the FNAME flag is set
            while f.read(1) != b'\x00':  # Skip the original file name
                pass
        if flg & 0x10:  # If the FCMT flag is set
            while f.read(1) != b'\x00':  # Skip the file comment
                pass
        if flg & 0x02:  # If the FHCRC flag is set
            f.seek(2, 1)  # Skip the header CRC

        # Read the DEFLATE stream
        deflate_data = bytearray()
        while True:
            byte = f.read(1)
            if not byte:
                break
            deflate_data.append(byte[0])

        # Remove the 4-byte CRC and ISIZE fields at the end of the gzip stream

        deflate_data = deflate_data[:-8]

        # Write the DEFLATE data to the output file
        with open(deflate_file_path, 'wb') as deflate_file:
            deflate_file.write(deflate_data)


def main():
    for file_name, generator in GENERATORS.items():
        file_path = get_data_path(file_name)
        # if data path has a directory, create it
        os.makedirs(os.path.dirname(file_path), exist_ok=True)
        generator[0](file_path, *generator[1:])

        gzip_file_path = get_data_path(get_gzip_name(file_name))
        compress_with_gzip(file_path, gzip_file_path)

        deflate_file_path = get_data_path(get_deflate_name(file_name))
        extract_deflate_data(gzip_file_path, deflate_file_path)

    manifest = {f: get_deflate_name(f) for f in GENERATORS}
    with open(get_data_path(MANIFEST_FILE), 'w') as f:
        json.dump(manifest, f)


if __name__ == '__main__':
    main()
