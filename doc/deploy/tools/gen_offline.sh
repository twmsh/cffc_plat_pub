#!/bin/bash

if [[ $# -ne 2 ]]; then
  echo "error, need 2 arguments: dst_dir repo_name "
  exit 1
fi

src_dir="/var/cache/apt/archives"
dst_dir="$(
  cd "$(dirname "$1")"
  pwd
)/$(basename "$1")"
repo_name=$2

function build_offline() {
  dst=$1
  repo=$2

  mkdir -p ${dst}/archives
  cp -a ${src_dir}/*.deb ${dst}/archives/

  cd ${dst}/archives/
  dpkg-scanpackages -m . /dev/null | gzip -c >Packages.gz

  cd ${dst}/
  echo "deb [trusted=yes] file://${dst}/archives ./" >${repo}.list
}

## 安装 dpkg-dev
# sudo apt-get install dpkg-dev

build_offline ${dst_dir} ${repo_name}

echo "done."
