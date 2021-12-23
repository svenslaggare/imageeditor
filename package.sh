#!/bin/bash
mkdir -p imageeditor
cp target/release/imageeditor imageeditor/imageeditor
cp -r content imageeditor/content
zip -r imageeditor.zip imageeditor/*
rm -rf imageeditor