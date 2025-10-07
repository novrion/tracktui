#!/bin/bash

COMMAND_TO_RUN="./tracktui"
STATE_FILE="$HOME/.tracktui_state"
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
