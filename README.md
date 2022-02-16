# krakenctl
Change the display on your Z73 Kraken from linux

## Disclaimer
This is alpha software, and may damage your device. Your cooler may stop functioning, be damaged, bricked stop working, which may in turn affect your other devices, most notably your CPU! Use this at your own risk. We take no responsibility for any damage to any of your devices or systems you run this on. 


### How to use
$ krakenctl [OPTIONS]

| Option      | Description | 
| :---        | :---        | 
| -b          | shows blank screen      | 
| -l          | shows liquid temperature   | 
| -c VAL      | shows value where VAL is a comma separated string of values between 0-100 e.g.:   |
|             | krakenctl -c 55
|             | krakenctl -c 33,45
| -s TXT      | shows subtitle where TXT is a comma separated string of 3 characters e.g.: |
|             |  krakenctl -s CPU |
|             |  krakenctl -s CPU,GPU |
| -r VAL      | sets brightness between 0-100 e.g. krakenctl -r 60 |
