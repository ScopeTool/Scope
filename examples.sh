

while true; do echo `cat /proc/cpuinfo | grep "cpu MHz" | head -1 | cut -d ':' -f 2`; sleep 1; done | scope 
