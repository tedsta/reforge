echo "Starting server ..."
bin/server &
sleep 3
echo "Starting first client ..."
bin/client localhost &
sleep 1
echo "Starting second client ..."
bin/client localhost &

sleep 1
read -p "Press enter to terminate. " answer

kill %1
kill %2
kill %3

