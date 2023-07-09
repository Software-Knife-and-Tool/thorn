import os
import sys
import subprocess

with open(sys.argv[1]) as f: test_source = f.readlines()

def runtest(test, expected):
    proc = subprocess.Popen(['../dist/runtime',
                             '-l../dist/core.l',
                             '-p',
                             '-e' + test],\
                            stdout=subprocess.PIPE,\
                            stderr=subprocess.PIPE)
    
    obtained = proc.stdout.read()[:-1].decode('utf8')
    err = proc.stderr.read()[:-1].decode('utf-8')

    proc.communicate()
        
    if proc.poll() != 0:
        print(f"{test}\t{expected}\t{err}\taborted")
    else:
        if obtained == expected:
            print(f"{test}\t{expected}\t{obtained}\tpassed")
        else:
            print(f"{test}\t{expected}\t{obtained}\tfailed")    

line_no = 0
for test in test_source:
    line_no += 1
    fields = test[:-1].split('\t')
    if len(fields) != 2:
        print(line_no, end=" !malformed test source: ")
        print(test, end="")
        continue

    test, expected = fields
    runtest(test, expected)
