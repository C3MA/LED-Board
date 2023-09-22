#!/bin/bash
# Wrapper script to update project and build project
#
#Set target IP address
IP=
# Path to this project
HOSTCLIENT=
cd $HOSTCLIENT
/usr/bin/pkill LEDboardClient
git pull
cargo build
$HOSTCLIENT/target/debug/LEDboardClient $IP
