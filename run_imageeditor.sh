#!/bin/bash
RUN_DIR=$(readlink -f $(dirname "$0"))
pushd $RUN_DIR &> /dev/null
./imageeditor_bin
popd &> /dev/null
