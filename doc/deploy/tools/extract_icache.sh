#!/bin/bash

set -e

if [[ $# -ne 1 ]]; then
  echo "error, need bmnnsdk2-bm1684_xxxx.tar.gz file"
  exit 1
fi

sdk_file=$1
sdk_tag=$(basename $sdk_file .tar.gz)

echo $sdk_tag
mkdir -p ./icache
tar xvfz ${sdk_file} -C ./icache ${sdk_tag}/bin/arm/test_update_fw ${sdk_tag}/bin/arm/bm168x_bmdnn_en_icache.bin ${sdk_tag}/bin/arm/bm168x_bmdnn_s_en_icache.bin

cd ./icache
mv ${sdk_tag}/bin/arm/test_update_fw ${sdk_tag}/bin/arm/bm168x_bmdnn_en_icache.bin ${sdk_tag}/bin/arm/bm168x_bmdnn_s_en_icache.bin .
rm -rf ${sdk_tag}

cd ../
tar cvfz icache.tgz icache
echo "done."
