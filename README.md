# LED-Board
## Project Page
[C3MA](https://www.ccc-mannheim.de/wiki/LED-Board)

## Controller 
* PlatformIO

### Hardware
* Arduino MEGA2560
* Ethernet shield

### Firmware
* PlatformIO based

see **platformio.ini**

## Client Software
stored in folder **/client**

### Dependency
* rust
* cargo

### Build
go to **/client**
* cargo build
* cargo run

### Deamon
Requires ''systemd''

Install by creating a link to this project
```
/etc/systemd/system# ln -s /home/c3ma/led-board/client/ledBoard.service ledBoard.service
systemctl daemon-reload
systemctl enable ledBoard.service
```
Start deamon with
```
systemctl start ledBoard.service
```
