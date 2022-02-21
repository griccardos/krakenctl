# krakenctl
Change the display on your Z73 Kraken

Krakenctl is an application written in rust, that allows users to change the display on their Z73 Kraken device.
Currently the manufacturer only provides the software for Windows that has the ability to show CPU/GPU, images etc.
Setting to liquid, or animation in Windows and then booting in linux works, however if you need to show CPU temps in linux, this is not currently possible.
This app allows you to update the display with what ever values you please (normally cpu temps).

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

It is recommended if using 2 amounts, to keep the amounts short, only use 2 digits and a degree symbol.
Also, amounts without decimals is preferred.

### Linux continuous update
Current best way is to loop a script that updates every few seconds and sends values to krakenctl.
For example if cputemp.sh is the script that returns the temperature, and you want updates every 1 second you'd use:
$ while true; do; krakenctl -v $(cputemp.sh); sleep 1; done;
or with gpu
$ while true; do; krakenctl -v "$(cputemp.sh),$(gpu.sh);CPU,GPU"; sleep 1; done;

### Current flaws
When using krakenctl in linux, when booting into windows, the USB device cannot be found. If you only use linux, probably not a problem, but if you dual boot, you can try the following ways to get it working on windows again. If anybody has any knowledge of why this happens, let me know.
- with computer off, remove the usb cable from the Kraken and wait a few seconds and replace
- turn off computer, AND remove plug, wait for 30 seconds (depends on motherboard), and replace


### Technical
krakenctl is written in rust, and uses the rusb crate which in turn uses libusb library.

### Roadmap
- [x] blank screen
- [x] liquid
- [x] values with subtitles
- [x] linux support
- [ ] windows support
- [ ] custom image
- [ ] custom animation
- [ ] getting USB not to lock in windows

### Contribute
If you find this application useful, or would like to make a contribution for continued development, you can buy me a coffee.
<a href="https://www.buymeacoffee.com/griccardos" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/default-orange.png" alt="Buy Me A Coffee" height="41" width="174"></a>
