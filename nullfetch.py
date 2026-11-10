#!/usr/bin/env python3
import platform
import getpass
import sys
import os
import subprocess

if "--help" in sys.argv:
    print("nullfetch - PC info")
    sys.exit()

#pcinfo:
user = getpass.getuser()
host = platform.node()
distro = platform.freedesktop_os_release()["PRETTY_NAME"]
kernel = platform.release()
term = os.environ.get("TERM", "?")

cpu = "?"
with open("/proc/cpuinfo") as f:
    for line in f:
        if line.startswith("model name"):
            cpu = line.split(":", 1)[1].strip()
            break

with open("/proc/meminfo") as f:
    mem = {}
    for line in f:
        key, value = line.split(":", 1)
        mem[key] = value.strip()

ram_total = int(mem["MemTotal"].split()[0]) // 1024 // 1024
ram_free = int(mem["MemAvailable"].split()[0]) // 1024 // 1024
ram = f"{ram_total - ram_free} / {ram_total} GB"

gpu = "?"
try:
    out = subprocess.run(["lspci"], capture_output=True, text=True).stdout
    for line in out.splitlines():
        if "VGA compatible controller" in line or "3D controller" in line:
            gpu = line.split(": ", 1)[1].split(" (rev")[0]
            break
except FileNotFoundError:
    pass


def printlogo():
    print(r"""        .--.
       |o_o |
       |:_/ |
      //   \ \
     (|     | )
    /'\_   _/`\
    \___)=(___/""")


def show(label, value):
    print(f"\033[1m{label}:\033[0m {value}")

def pcinfo():
    show("USER",user)
    show("HOST",host)
    show("OS", distro)
    show("kernel", kernel)
    show("terminal", term)
    show("CPU", cpu)
    show("GPU", gpu)
    show("RAM", ram)

#line1
print("--------------------------")

#pictureprint:
printlogo()

#space
print("                   ")

#pcinfoprint:
pcinfo()

#line2
print("--------------------------")
