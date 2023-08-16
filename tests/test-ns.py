import json
import sys
import subprocess

ns = sys.argv[1]
base = sys.argv[2]

with open(base + '/' + ns + '/tests') as f: group_list = f.readlines()

def runtest(line, test, expected):
    if ns == 'mu':
        proc = subprocess.Popen(['../dist/runtime', '-p', '-e' + test],\
                                stdout=subprocess.PIPE,\
                                stderr=subprocess.PIPE)
    if ns == 'core':
        proc = subprocess.Popen(['../dist/runtime',
                                 '-l../dist/core.l',
                                 '-p',
                                 '-e' + test],\
                                stdout=subprocess.PIPE,\
                                stderr=subprocess.PIPE)
    
    obtained = proc.stdout.read()[:-1].decode('utf8')
    err = proc.stderr.read()[:-1].decode('utf-8')

    proc.communicate()

    aborted = False if proc.poll() == 0 else True
    passed = True if obtained == expected else False
    result = { 'expect': expected, 'obtain': obtained }

    return { 'line': line, 'abort': aborted, 'pass': passed, 'result': result }

ns_results = []
for group in group_list:
    results = []
    with open(base + '/' + ns + '/' + group[:-1]) as f: test_source = f.readlines()
    
    line = 0
    for test in test_source:
        line += 1
        fields = test[:-1].split('\t')
        if len(fields) != 2:
            results.append({ 'line': line, 'test syntax': fields })
            continue

        test, expected = fields
        results.append(runtest(line, test, expected))

    ns_results.append({'group': group[:-1], 'results': results})

print(json.dumps({ 'ns': sys.argv[1], 'results': ns_results }))
