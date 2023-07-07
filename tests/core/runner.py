import os
import sys
import subprocess

with open(sys.argv[1]) as f: test_source = f.readlines()

def runtest(test, expected):
    proc = subprocess.Popen(['../dist/runtime',
                             '-l../src/core/core.l',
	                     '-l../src/core/closures.l',
	                     '-l../src/core/fixnums.l',
	                     '-l../src/core/read-macro.l',
	                     '-l../src/core/read.l',
	                     '-l../src/core/sequences.l',
	                     '-l../src/core/symbol-macro.l',
	                     '-l../src/core/symbols.l',
	                     '-l../src/core/vectors.l',
                             '-l../src/core/compile.l',
                             '-l../src/core/exceptions.l',
                             '-l../src/core/format.l',
                             '-l../src/core/lambda.l',
                             '-l../src/core/lists.l',
                             '-l../src/core/load.l',
                             '-l../src/core/macro.l',
                             '-l../src/core/parse.l',
                             '-l../src/core/perf.l',
                             '-l../src/core/quasiquote.l',
                             '-l../src/core/streams.l',
                             '-l../src/core/strings.l',
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
    fields=test[:-1].split('\t')
    if len(fields) != 2:
        print(line_no, end=" !malformed test source: ")
        print(test, end="")
        continue

    test, expected = fields
    runtest(test, expected)
