import struct, zlib, subprocess, sys
from pathlib import Path
from conftest import run

def chunk(kind,data):
    return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)

def png_bytes(w,h):
    rows=[]
    for y in range(h):
        row=bytearray([0])
        for x in range(w):
            row += bytes((x*30%255,y*30%255,100,255))
        rows.append(bytes(row))
    raw=b''.join(rows)
    ihdr=struct.pack('>IIBBBBB',w,h,8,6,0,0,0)
    return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',ihdr)+chunk(b'IDAT',zlib.compress(raw))+chunk(b'IEND',b'')


def test_cli_image_blocks(tmp_path):
    p = tmp_path/'a.png'
    p.write_bytes(png_bytes(8,4))
    r = run('--image', str(p), '--image-mode','blocks','--no-logo','--only','user','--no-color')
    assert r.returncode == 0
    assert 'USER:' in r.stdout
    assert '▀' in r.stdout or ' ' in r.stdout
