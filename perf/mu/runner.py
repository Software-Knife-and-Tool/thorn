import sys
import subprocess

ntests = sys.argv[1]
with open(sys.argv[2]) as f: test_source = f.readlines()

def storage(test):
    proc = subprocess.Popen(['../dist/thorn-local.sh',
                             '--pipe',
                             '--eval=(core::storage-delta (:lambda ()' + test + ') :nil)'],\
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
