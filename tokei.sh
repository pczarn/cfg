#!/usr/bin/bash
tokei cfg{-grammar,-regexp,-load,-history,-sequence,-symbol,-symbol-bit-matrix,-predict-sets,}/src

if [ $? -eq 127 ]; then
  echo "please run cargo install tokei (exit code 127)"
fi

