#!/usr/bin/env bash

# processes to run in each pane
INFLUX="influxd --reporting-disabled"
BACKEND="source .env; cargo run -p backend"
FRONTEND="npm --prefix frontend run dev"
FIREFOX="sleep 1;firefox http://localhost:3002 http://localhost:5173"

sudo systemctl start grafana.service

SESSION="amber"
tmux kill-session -t "$SESSION" 2>/dev/null

# Split into 2x2 grid of panes
tmux new-session -d -s "$SESSION" -n "main"
tmux split-window -t "$SESSION:main" -h
tmux split-window -t "$SESSION:main.1" -v
tmux split-window -t "$SESSION:main.3" -h

# even out the pane sizes
tmux select-layout -t "$SESSION:main" tiled

# send commands to each pane
tmux send-keys -t "$SESSION:main.1" "$INFLUX" Enter
tmux send-keys -t "$SESSION:main.2" "$BACKEND" Enter
tmux send-keys -t "$SESSION:main.3" "$FRONTEND" Enter
tmux send-keys -t "$SESSION:main.4" "$FIREFOX" Enter

tmux select-pane -t "$SESSION:main.3"
tmux attach-session -t "$SESSION"