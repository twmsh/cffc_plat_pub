#!/bin/bash

if [[ $# -ne 1 ]]; then
  echo "error, need repo deb file"
  exit 1
fi

deb_file=$1

if [[ ! -e ${deb_file} ]]; then
  echo "error, ${deb_file} not found"
  exit 1
fi

sudo dpkg -i "${deb_file}"
sudo apt-get update

sudo apt-get -y install libboost-regex1.62.0
sudo apt-get -y install libgoogle-glog0v5

sudo apt-get -y install fleecer
sudo apt-get -y install server
sudo apt-get -y install fleecer-auth-tools
sudo apt-get -y install deepface-client
sudo apt-get -y install remoteforward

##
#sudo apt-get -y install libmongo mongodb-server

#sudo mkdir -p /run/mongodb
#sudo chown -R mongodb.mongodb /run/mongodb

# 开机⾃启
sudo systemctl enable mongodb
sudo systemctl enable license-server
sudo systemctl enable fleecer
sudo systemctl enable server

sudo systemctl start mongodb
sudo systemctl start license-server
sudo systemctl start server
sudo systemctl start fleecer

echo "need activate license"
echo "example: sudo activate"
