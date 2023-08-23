import json
import sys
from datetime import datetime

with open(sys.argv[1]) as f: test_results = json.load(f)

ns = test_results['ns']
test_name = test_results['test']
print(f'Test Summary: {ns}/{test_name}')
print('-----------------------')

test_total = 0
test_fails = 0
test_passes = 0
test_aborts = 0

for test in test_results['results']:
    test_total += 1
    test_passes += 1 if test['pass'] else 0
    test_aborts += 1 if test['abort'] else 0
    test_fails += 1 if not test['pass'] and not test['abort'] else 0

    status = 'abort' if test['abort'] and not test['pass'] else 'fail'

    result = test['result']    
    if test['abort'] or not test['pass']:
        line = test['line']
        test = test['test']
        expect = result['expect']
        obtain = result['obtain']
        ftest = (test[:27] + '...') if len(test) > 30 else test
        ferr = (obtain[:12] + '...') if len(obtain) > 15 else obtain
        fexpect = (expect[:12] + '...') if len(expect) > 15 else expect
        print(f'{line:>3} {ftest:<30} {fexpect:<15} {ferr:<15} {status:<8}')

print('-----------------------')
print(f'{ns:<11}', end='')
print(f'passed: {test_passes:<7}', end='')
print(f'total: {test_total:<9}', end='')
print(f'failed: {test_fails:<9}', end='')
print(f'aborted: {test_aborts:<10}')
print()
