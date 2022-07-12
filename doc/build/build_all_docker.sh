#!/bin/bash

set -e

os_target="arm64"
#cargo clean
#export TARGET_CC=aarch64-linux-musl-gcc
rust_target="aarch64-unknown-linux-musl"

## project name
workspace="cffc_plat"
proj_path="/rust_projects/${workspace}"
build_path="${proj_path}/target/${rust_target}/release"
dst_path="/rust_projects/${workspace}/dist"

app_build=$(date "+%Y/%m/%d %H:%M:%S")
app_version="0.1.0"
ts=$(date "+%m%d%H%M")
release_file="cffc_plat-${os_target}-${app_version}-${ts}.tgz"

function build_worker() {
  src=${proj_path}/bm_worker
  dst=${dst_path}/${workspace}/bm_worker

  rm -rf ${dst}
  mkdir -p ${dst}/logs

  cp -a ${src}/static ${dst}/
  cp -a ${src}/views ${dst}/
  cp -a ${src}/cfg.json ${dst}/
  cp -a ${src}/camera.json ${dst}/

  cargo build --release --target ${rust_target} --bin bm_worker
  cp -a ${build_path}/bm_worker ${dst}/
}

function build_imp() {
  src=${proj_path}/bm_imp
  dst=${dst_path}/${workspace}/bm_imp

  rm -rf ${dst}
  mkdir -p ${dst}

  cp -a ${src}/cfg.json ${dst}/

  cargo build --release --target ${rust_target} --bin bm_imp
  cp -a ${build_path}/bm_imp ${dst}/
}

function build_tool() {
  src=${proj_path}/bm_tool
  dst=${dst_path}/${workspace}/bm_tool

  rm -rf ${dst}
  mkdir -p ${dst}

  cp -a ${src}/create_src.json ${dst}/
  cargo build --release --target ${rust_target} --bin bm_tool
  cp -a ${build_path}/bm_tool ${dst}/
}

function build_install() {
  src=${proj_path}
  dst=${dst_path}/${workspace}

  rm -rf ${dst}/install
  mkdir -p ${dst}/install/tools
  mkdir -p ${dst}/df_imgs

  cp -a ${src}/doc/deploy/cfbm.db ${dst}/

  cp -a ${src}/doc/deploy/bm_worker.service ${dst}/install/
  cp -a ${src}/doc/deploy/logrotate.conf ${dst}/install/
  cp -a ${src}/doc/deploy/install_bm_app.sh ${dst}/install/
  chmod +x ${dst}/install/install_bm_app.sh

  cp -a ${src}/doc/deploy/tools/clean_for_boxapp.sh ${dst}/install/tools/
  cp -a ${src}/doc/deploy/tools/install_bm_api.sh ${dst}/install/tools/
  cp -a ${src}/doc/deploy/tools/install_bm_api_offline.sh ${dst}/install/tools/
  cp -a ${src}/doc/deploy/tools/update_se5.sh ${dst}/install/tools/
  cp -a ${src}/doc/deploy/tools/gen_offline.sh ${dst}/install/tools/
  cp -a ${src}/doc/deploy/tools/before_install.sh ${dst}/install/tools/
  cp -a ${src}/doc/deploy/tools/after_install.sh ${dst}/install/tools/

  chmod +x ${dst}/install/tools/clean_for_boxapp.sh
  chmod +x ${dst}/install/tools/install_bm_api.sh
  chmod +x ${dst}/install/tools/install_bm_api_offline.sh
  chmod +x ${dst}/install/tools/update_se5.sh
  chmod +x ${dst}/install/tools/gen_offline.sh
  chmod +x ${dst}/install/tools/before_install.sh
  chmod +x ${dst}/install/tools/after_install.sh
}

function build_tgz() {
  work_dir=${dst_path}
  tar_file=${release_file}

  cd ${work_dir} || return
  tar cfz ${tar_file} ${workspace}
  echo "${work_dir}/${tar_file}"
}

build_worker
build_imp
build_tool
build_install
build_tgz

echo "done."
