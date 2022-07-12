#!/bin/bash

CURDIR=$(pwd)
BASEDIR=$(dirname "$CURDIR")
echo ${BASEDIR}

DST_USER="user"

## stop it
sudo systemctl stop bm_worker

cd ${BASEDIR}/install

## replace INSTDIR

sed -i -e "s#INSTDIR#$BASEDIR#g" ${BASEDIR}/bm_worker/cfg.json

sed -e "s#INSTDIR#$BASEDIR#g" -e "s#User=linaro#User=$DST_USER#g" bm_worker.service | sudo tee /lib/systemd/system/bm_worker.service >/dev/null
sudo systemctl daemon-reload
sudo systemctl enable bm_worker
sed -e "s#INSTDIR#$BASEDIR#g" -e "s#su linaro linaro#su $DST_USER $DST_USER#g" logrotate.conf | sudo tee /etc/logrotate.d/bm_worker >/dev/null

cd ${BASEDIR}/bm_tool
chmod +x bm_tool
./bm_tool reset --db file:${BASEDIR}/cfbm.db?_loc=auto --img_dir ${BASEDIR}/df_imgs --url_a http://localhost:7001 --url_r http://localhost:7002

sudo systemctl start bm_worker

echo "done."
