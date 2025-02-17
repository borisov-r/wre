import esp
import network
import time
import ubinascii

wlan_id = "A1_2621"
wlan_pass = "2BCH7233EL"

mac = ubinascii.hexlify(network.WLAN().config('mac'),':').decode()
print("MAC: " + mac)

wlan = network.WLAN(network.STA_IF)
wlan.active(True)
while wlan.status() is network.STAT_CONNECTING:
    time.sleep(1)
while not wlan.isconnected():
    wlan.connect(wlan_id, wlan_pass)
print("Connected... IP: " + wlan.ifconfig()[0])

#import esp
#import network
import machine
from machine import Pin
import os
#import time #<--- if used typical connection code from example
#import ubinascii #<--- if used typical connection code from example
import utils #<--- if user utils
from micropyserver import MicroPyServer

# setup interrupts

a = Pin(21, Pin.IN, Pin.PULL_UP)
b = Pin(22, Pin.IN, Pin.PULL_UP)

def callback(pin):
    print(pin, type(pin))
    print(pin.value(), type(pin.value()))
    if pin.value() == 21:
        print("Phase A")
    elif pin.value() == 22:
        print("Phase B")
    else:
        print(f"{pin}")

# def phase_a(pin):
#     print(f"Phase A: {pin}")
# 
# def phase_b(pin):
#     print(f"Phase B: {pin}")
        
a.irq(trigger=Pin.IRQ_FALLING, handler=callback)
b.irq(trigger=Pin.IRQ_FALLING, handler=callback)

#from micropyserver import MicroPyServer

''' there should be a wi-fi connection code here '''

def hello_world(request):
    ''' request handler '''
    server.send(f"HELLO WORLD!</br>Phase A: {a},</br>Phase B: {b}")

server = MicroPyServer()
''' add route '''
server.add_route("/", hello_world)
''' start server '''
server.start()
