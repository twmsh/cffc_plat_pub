#!/bin/bash

## 1. enable icache
## 2. some tools

## 1
icache_srcdir="/data/bak/icache"
icache_dstdir="/system/bin"
rc_file="/etc/rc.local"

if [[ ! -d "${icache_srcdir}" ]]; then
  echo "${icache_srcdir} not found"
  exit 1
fi

chmod +x ${icache_srcdir}/test_update_fw
sudo cp ${icache_srcdir}/test_update_fw ${icache_dstdir}/
sudo cp ${icache_srcdir}/bm168x_bmdnn_en_icache.bin ${icache_dstdir}/
sudo cp ${icache_srcdir}/bm168x_bmdnn_s_en_icache.bin ${icache_dstdir}/

icache_txt=$(grep "test_update_fw" ${rc_file})
if [[ -n "${icache_txt}" ]]; then
  echo "find test_update_fw, skip"
else
  echo "add icache to rc.local"
  sed "$ i cd ${icache_dstdir}; ./test_update_fw ./bm168x_bmdnn_en_icache.bin ./bm168x_bmdnn_s_en_icache.bin 0" <${rc_file} | sudo tee ${rc_file} >/dev/null
fi

## 2
sudo apt-get -y install sqlite3 screen iftop htop unrar lrzsz

echo ""
echo "done."
