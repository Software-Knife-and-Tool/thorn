import json
import sys
from datetime import datetime

with open(sys.argv[1]) as f: test_results = json.load(f)

print(f'Test Summary:')
print('-----------------------')

ns = test_results['ns']
groups = test_results['results']

test_total = 0
test_fails = 0
test_aborts = 0
for group in groups:
    group_label = group['group']
    results = group['results']

    total = 0
    passed = 0
    aborted = 0
    for result in results:
        total += 1
        passed += 1 if result['pass'] else 0
        aborted += 1 if result['abort'] else 0

    failed = total - passed - aborted
    print(f'{ns:<10} {group_label:<14} total: {total:<8} passed: {passed:<8} failed: {failed:<8} aborted: {aborted:<8}')
    test_total += total
    test_fails += failed
    test_aborts += aborted

test_passes = test_total - (test_fails + test_aborts)
print('-----------------------')
print(f'{ns:<11}', end='')
print(f'passed: {test_passes:<7}', end='')
print(f'total: {test_total:<9}', end='')
print(f'failed: {test_fails:<9}', end='')
print(f'aborted: {test_aborts:<10}')
print()
