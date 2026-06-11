#!/bin/bash

PROJECT_DIR="$HOME/Projects/noisy-skyy"

kitty \
  --detach \
  --session <(
    cat <<EOF
new_tab
cd $PROJECT_DIR
launch cargo run --release -- --listen

new_tab
cd $PROJECT_DIR
launch bash -lc 'sleep 1; cargo run --release -- -s testing/sample1.txt'
EOF
)
