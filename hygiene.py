
# This script runs all tests, warns about
# code complexity, and ensures developers can
# build features without having to slow down
# to ensure high-quality code is used.

# Please run this script often and apply fixes to
# issues, failing tests, and clippy lints.

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
