#!/bin/bash -ex
# SPDX-License-Identifier: Apache-2.0

if [ "$(objdump -p $1 |sed -ne 's/.*SONAME \+\(libnispor.\+\)/\1/p')" \
    != "libnispor.so.1" ];then
    exit 1
fi
