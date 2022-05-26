#!/bin/false "This script should be sourced in a shell, not executed directly"
killall -e '$binary_name' >/dev/null 2>&1 || true
