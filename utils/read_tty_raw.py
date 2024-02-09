import sys

fp_out = open("/dev/ttys011", "rb")

while True:
    print(''.join('{:02x}'.format(x) for x in fp_out.read(1)), end=' ')
    sys.stdout.flush()
