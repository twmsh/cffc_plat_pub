#!/bin/bash

## 0. modify timezone
## 1. stop/disable services not need
## 2. remvoe files/dir not need
## 3. remove mongodb(maybe unmatch version)
## 4. modify apt repo

## 0
sudo timedatectl set-timezone Asia/Shanghai
sudo systemctl enable systemd-timesyncd
sudo systemctl restart systemd-timesyncd

## 1
echo "disable/stop other services ..."
sudo systemctl disable SophonGate
sudo systemctl disable SophonFaceCapCam
sudo systemctl disable SophonHistoryEvent
sudo systemctl disable SophonMongoQuota
sudo systemctl disable nginx
sudo systemctl disable indicator_light
sudo systemctl disable SophonHDMI
sudo systemctl disable docker
sudo systemctl disable bm_server
sudo systemctl disable SophonHDMIStatus

sudo systemctl stop SophonGate
sudo systemctl stop SophonFaceCapCam
sudo systemctl stop SophonHistoryEvent
sudo systemctl stop SophonMongoQuota
sudo systemctl stop nginx
sudo systemctl stop indicator_light
sudo systemctl stop SophonHDMI
sudo systemctl stop docker
sudo systemctl stop bm_server
sudo systemctl stop SophonHDMIStatus

## 2
echo "remove /system/SophonFaceSDK ..."
sudo rm -rf /system/SophonFaceSDK

echo "remove /system/data/sophon_gate ..."
sudo rm -rf /system/data/sophon_gate

## 3
echo "remove mongodb ..."
sudo systemctl stop mongodb
sudo systemctl disable mongodb

sudo apt-get -y purge libmongo
sudo apt-get -y purge mongodb-server

sudo rm -rf /var/log/mongodb
sudo rm -rf /data/db
sudo apt-get -y autoremove

## 4
ts=$(date "+%m%d%H%M%S")
apt_file="/etc/apt/sources.list"
apt_bak="${apt_file}.${ts}"

echo "modify apt sources ..."
sudo cp ${apt_file} ${apt_bak}

cat <<EOF | sudo tee ${apt_file} >/dev/null
deb https://mirrors.ustc.edu.cn/debian/ stretch main contrib non-free
deb-src https://mirrors.ustc.edu.cn/debian/ stretch main contrib non-free
deb https://mirrors.ustc.edu.cn/debian/ stretch-updates main contrib non-free
deb-src https://mirrors.ustc.edu.cn/debian/ stretch-updates main contrib non-free
deb https://mirrors.ustc.edu.cn/debian/ stretch-backports main contrib non-free
deb-src https://mirrors.ustc.edu.cn/debian/ stretch-backports main contrib non-free
deb https://mirrors.ustc.edu.cn/debian-security/ stretch/updates main contrib non-free
deb-src https://mirrors.ustc.edu.cn/debian-security/ stretch/updates main contrib non-free
EOF

sudo apt-get update
echo ""
echo "done."
