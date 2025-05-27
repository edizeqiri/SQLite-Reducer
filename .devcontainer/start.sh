#!/bin/bash
# start.sh

# Start SSH in the background
/usr/sbin/sshd

# Start Jupyter Notebook in foreground
exec jupyter notebook --ip=0.0.0.0 --port=8888 --no-browser --allow-root
