import sys
import subprocess

ntests = sys.argv[1]
with open(sys.argv[2]) as f: test_source = f.readlines()

def storage(test):
    proc = subprocess.Popen(['../dist/runtime',
                             '-l../dist/core.l',
                             '-p',
                             '-e(core::storage-delta (:lambda ()' + test + ') :nil)'],\
                            stdout=subprocess.PIPE,\
                            stderr=subprocess.PIPE)

    storage = proc.stdout.read()[:-1].decode('utf8')
    err = proc.stderr.read()[:-1].decode('utf-8')

    proc.communicate()
        
    if proc.poll() != 0:
        print(f'mu:,{test},{err}')
    else:
        print(f'mu:,{sys.argv[2]},{storage}', end="")

def timing(test):
    proc = subprocess.Popen(['../dist/runtime',
                             '-l../dist/core.l',
                             '-p',
                             '-e(core::time-delta (:lambda ()' + test + ') :nil)'],\
                            stdout=subprocess.PIPE,\
                            stderr=subprocess.PIPE)
    
    timing = proc.stdout.read()[:-1].decode('utf8')
    err = proc.stderr.read()[:-1].decode('utf-8')

    proc.communicate()
        
    if proc.poll() != 0:
        print(f'mu:,{test},{err}')
    else:
        print(f',{timing}', end="")

for test in test_source:
    storage(test[:-1])

    for n in range(int(ntests)):
        timing(test[:-1])

    print()
