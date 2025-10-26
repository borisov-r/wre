#import network
#import webrepl
#webrepl.start()

#ssid = 'abkant'                  #Set your own 
#password = '12345678'            #Set your own password

#ssid = 'A1_2621'                  #Set your own 
#password = '2BCH7233EL'            #Set your own password

#ap = network.WLAN(network.AP_IF)
#ap.active(True)
#ap.config(essid=ssid, authmode=3, password=password)
#ap.config(max_clients=2)         # set how many clients can connect to the network
#ap.active(True)                  # activate the interface

#while ap.active() == False:
#  pass
#print('Connection is successful')
#print(ap.ifconfig())

#sta = network.WLAN(network.STA_IF)
#sta.active(True)
#sta.connect(ssid, password)

#while not sta.isconnected():
#    pass

#print('Connected to Wi-Fi. IP:', sta.ifconfig()[0])
