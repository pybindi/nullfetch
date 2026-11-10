#!/usr/bin/env python3
import platform
import getpass

#pcinfo:
user = getpass.getuser()
host = platform.node()
distro = platform.freedesktop_os_release()["PRETTY_NAME"]
kernel = platform.release()
arct = platform.machine()


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
    show("architecture", arct)


#pictureprint:
printlogo()

#pcinfoprint:
pcinfo()

#space
print("            ")

#line
print("--------------------------")
