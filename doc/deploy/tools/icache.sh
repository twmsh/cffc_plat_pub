#!/bin/bash

rc_file="/etc/rc.local"
dst_dir="/system/bin"

chmod +x test_update_fw
sudo cp test_update_fw ${dst_dir}/
sudo cp bm168x_bmdnn_en_icache.bin ${dst_dir}/
sudo cp bm168x_bmdnn_s_en_icache.bin ${dst_dir}/

## cd /system/bin; ./test_update_fw ./bm168x_bmdnn_en_icache.bin ./bm168x_bmdnn_s_en_icache.bin 0

###  增加rc-local
driver_txt=$(grep "test_update_fw" ${rc_file})
if [[ -n "${driver_txt}" ]]; then
  echo "find test_update_fw, skip"
else
  echo "add icache to rc.local"
  sed "$ i cd ${dst_dir}; ./test_update_fw ./bm168x_bmdnn_en_icache.bin ./bm168x_bmdnn_s_en_icache.bin 0" <${rc_file} | sudo tee ${rc_file} >/dev/null
fi
