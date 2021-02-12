
# This script runs all tests, warns about
# code complexity, and ensures developers can
# build features without having to slow down
# to ensure high-quality code is used.

# Please run this script often and apply fixes to
# issues, failing tests, and clippy lints.

# Misc. setup commands, may be necessary depending on
# how the dev has setup their OS:
# 
#    rustup component add clippy
#    cargo install cargo-fuzz
# 

import os
import sys
import subprocess


def main(args=sys.argv):
  subprocess.run([
    'cargo', 'clippy'
  ])
  subprocess.run([
    'cargo', 'test'
  ])

if __name__ == '__main__':
  main()
