#!/bin/bash

# Sleep for 5 hours (5 * 3600 seconds)
sleep $((5*3600))

# Execute the command
claude --permission-mode acceptEdits -c "go on"
