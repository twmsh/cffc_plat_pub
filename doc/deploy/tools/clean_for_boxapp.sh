#!/bin/bash

###  增加rc-local中的 load_driver
driver_txt=$(grep "load_driver.sh" /etc/rc.local)
if [[ -n "${driver_txt}" ]]; then
  echo "find driver.sh, skip"
else
  echo "add driver.sh to rc.local"
  cat /etc/rc.local | sed '$ i sh /system/data/load_driver.sh' | sudo tee /etc/rc.local >/dev/null
fi

## 停止无关服务
sudo systemctl stop bm_server
sudo systemctl stop npm
sudo systemctl stop matcher
sudo systemctl stop npm_support
sudo systemctl stop npm_support_1
sudo systemctl stop nginx
sudo systemctl stop docker
sudo systemctl stop SophonGate
sudo systemctl stop SophonFaceCapCam
sudo systemctl stop vendor
sudo systemctl stop SophonMongoQuota

sudo systemctl disable bm_server
sudo systemctl disable npm
sudo systemctl disable matcher
sudo systemctl disable npm_support
sudo systemctl disable npm_support_1
sudo systemctl disable nginx
sudo systemctl disable docker
sudo systemctl disable SophonGate
sudo systemctl disable SophonFaceCapCam
sudo systemctl disable vendor
sudo systemctl disable SophonMongoQuota

## 停止deepface-client / remoteforward服务
sudo systemctl stop deepface-client
sudo systemctl disable deepface-client

sudo systemctl stop remoteforward
sudo systemctl disable remoteforward

## 清除mongodb数据
sudo systemctl stop mongodb
sudo rm -rf /system/db/*
sudo systemctl start mongodb
sudo systemctl enable mongodb

##
echo "done. should reboot"
