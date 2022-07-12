#!/bin/bash

tool_path="/Users/tom/develop/RustProjects/dbmodelgen/target/release"
tool="sqlite_gen"

db="file:../deploy/cfbm.db"
dst_file="../../bm_worker/src/dao/model.rs"
pkg_name="cffc_base::db::dbop"

${tool_path}/${tool} --pkg "${pkg_name}" --db "${db}" --file "${dst_file}"
