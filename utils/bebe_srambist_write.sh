sudo python3 bebe_host.py --no_wait --addr 1000 --wdata 32 --wlen 8
sudo python3 bebe_host.py --no_wait --addr 1008 --wdata FFFFFFFFFFFFFFFF --wlen 8
sudo python3 bebe_host.py --no_wait --addr 1010 --wdata FFFFFFFFFFFFFFFF --wlen 8
sudo python3 bebe_host.py --no_wait --addr 1018 --wdata FFFFFFFFFFFFFFFF --wlen 8
sudo python3 bebe_host.py --no_wait --addr 1020 --wdata 3 --wlen 8
sudo python3 bebe_host.py --no_wait --addr 1028 --wdata 0 --wlen 8
sudo python3 bebe_host.py --no_wait --addr 1038 --wdata 0 --wlen 8
sudo python3 bebe_host.py --no_wait --addr 1180 --wdata FFFFFFFFFFFFFFFF --wlen 8

sudo python3 bebe_host.py --no_wait --addr 1000 --rlen 8
sudo python3 bebe_host.py --no_wait --addr 1008 --rlen 8
sudo python3 bebe_host.py --no_wait --addr 1010 --rlen 8
sudo python3 bebe_host.py --no_wait --addr 1018 --rlen 8
sudo python3 bebe_host.py --no_wait --addr 1020 --rlen 8
sudo python3 bebe_host.py --no_wait --addr 1028 --rlen 8
sudo python3 bebe_host.py --no_wait --addr 1038 --rlen 8
sudo python3 bebe_host.py --no_wait --addr 1180 --rlen 8
