# krakenctl
Change the display on your Z73 Kraken from linux

### Disclaimer
This is alpha software, and may damage your device. Your cooler may stop functioning, be damaged, bricked, or stop working, which may in turn affect your other devices, most notably your CPU! Use this at your own risk. We take no responsibility for any damage to any of your devices or systems you run this on. 


### How to use
$ krakenctl [OPTIONS]

| Option      | Description | 
| :---        | :---        | 
| -b          | shows blank screen      | 
| -l          | shows liquid temperature   | 
| -v Valuestring      | shows value(s) and or subtitles (see below for examples)    |
| -r brightness      | sets brightness between 0-100 e.g. krakenctl -r 60 |

### Valuestring
Made up of 2 parts, separated by a semicolon:
- value(s)
- subtitle(s)
In addition, each of these can be optionally separated by a comma to display 2 values.
Remember if using a semicolon, you may need to use quotes to surround the Valuestring
Examples:

| Desc | value |
| :--- | :--- |
| 1 amount only | 45° |
| 2 amounts | 45°,34° |
| 1 amount with subtitle | "45°;CPU" |
| 2 amounts with subtitles | "33°,45°;CPU,GPU" |

### Contribute
If you find this application useful, or would like to make a contribution for continued development, you can buy me a coffee.
<a href="https://www.buymeacoffee.com/griccardos" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/default-orange.png" alt="Buy Me A Coffee" height="41" width="174"></a>
