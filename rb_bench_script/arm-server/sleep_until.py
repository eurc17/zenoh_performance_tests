#!/usr/bin/env python3
import time
import sys

diff = max(float(sys.argv[1]) - time.time(), 0.0)
time.sleep(diff)
