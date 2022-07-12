#!/bin/bash

## install with offline package

if [[ $# -ne 2 ]]; then
  echo "error, need repo deb and offline deb files"
  exit 1
fi

repo_deb_file=$1
offline_deb_file=$2

if [[ ! -e ${repo_deb_file} ]]; then
  echo "error, ${repo_deb_file} not found"
  exit 1
fi

if [[ ! -e ${offline_deb_file} ]]; then
  echo "error, ${offline_deb_file} not found"
  exit 1
fi

sudo dpkg -i "${repo_deb_file}"
sudo dpkg -i "${offline_deb_file}"

##
sudo sed -i 's/^\([a-z]\)/#\1/g' /etc/apt/sources.list.d/linaro-overlay-obs.list
sudo sed -i 's/^\([a-z]\)/#\1/g' /etc/apt/sources.list.d/nodesource.list

sudo apt-get update

sudo apt-get -y install deepface-models
sudo apt-get -y install facebmapi-client
sudo apt-get -y install facebmapi-server
sudo apt-get -y install facebmapi-auth-tools

# 开机⾃启
sudo systemctl enable mongodb
sudo systemctl enable client
sudo systemctl enable server
sudo systemctl enable license-server

###  增加rc-local中的 load_driver
driver_txt=$(grep "load_driver.sh" /etc/rc.local)
if [[ -n "${driver_txt}" ]]; then
  echo "find driver.sh, skip"
else
  echo "add driver.sh to rc.local"
  sed '$ i sh /system/data/load_driver.sh' </etc/rc.local | sudo tee /etc/rc.local >/dev/null
fi

#开机关闭
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

## 其他可选工具
sudo apt-get -y install sqlite3 screen iftop htop unrar lrzsz

echo "need activate license"
echo "example: sudo activate"
