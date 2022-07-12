#!/bin/bash

echo ">>>>> starting update..."

# kernel
sudo cp emmcboot.itb /boot/
if [ $? == 0 ]; then
  echo "emmcboot.itb copyed."
else
  echo "emmcboot.itb copy failed!!!!!!!"
  exit 1
fi

# system.tgz
echo ">>>>> system.tgz upgrade starting..."
sudo tar xzf system.tgz -C /system/
if [ $? == 0 ]; then
  echo "uncompress system.tgz to /system succeed"
  sudo sync
  echo "upgrade done will reboot"
  sleep 10
  sudo reboot
else
  echo "system.tgz upgrade failed!!!!!!!"
  exit 1
fi
