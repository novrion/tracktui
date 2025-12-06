#!/bin/bash

COMMAND_TO_RUN="./bin/tracktui startup"
STATE_FILE="./.last_run"
TODAY=$(date +%Y-%m-%d)

# Read state
LAST_RUN_DATE=""
if [ -f "$STATE_FILE" ]; then
  LAST_RUN_DATE=$(cat "$STATE_FILE")
fi

# Eval cmd & write state
if [ "$TODAY" != "$LAST_RUN_DATE" ]; then
  eval $COMMAND_TO_RUN
  echo "$TODAY" >"$STATE_FILE"
fi
