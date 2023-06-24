#!/bin/bash
# Run fly logs, pipe to sed to remove ANSI codes, then pipe to sed to remove line prefixes

fly logs | sed -r "s/\x1B\[([0-9]{1,3}(;[0-9]{1,2})?)?[mGK]//g" | sed -E -e 's/[0-9T:-]*Z\sapp\[[a-fA-F0-9]+\]\s[a-z]{3}\s\[[a-z]+\]//' | bunyan
