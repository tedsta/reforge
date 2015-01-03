kill $(ps aux | grep "bin\/server" | awk '{print $2}')
