#!/bin/bash
mkdir -p imageeditor
cp target/release/imageeditor imageeditor/imageeditor_bin
cp run_imageeditor.sh imageeditor/imageeditor
cp -r content imageeditor/content