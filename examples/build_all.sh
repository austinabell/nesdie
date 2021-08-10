#!/bin/bash
set -e

echo $(rustc --version)
pushd $(dirname ${BASH_SOURCE[0]})

for d in $(ls -d */); do
    echo building $d;
    (cd "$d"; ./build.sh);
done

popd