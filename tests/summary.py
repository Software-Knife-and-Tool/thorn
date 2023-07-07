import sys
from datetime import datetime

with open(sys.argv[1]) as f: test_results = f.readlines()

print(f'Test Summary:')
print('-----------------------')

test_total = 0
test_fails = 0
test_aborts = 0
for test in test_results:
    fields = test[:-1].split(',')
    label, test, total, passed, failed, aborted = fields
    
    print(f'{label:<10} {test:<14} total: {total:<8} passed: {passed:<8} failed: {failed:<8} aborted: {aborted:<8}')
    test_total += int(total)
    test_fails += int(failed)
    test_aborts += int(aborted)

test_passes = test_total - (test_fails + test_aborts)
print('-----------------------')
print(f'{label:<11}', end='')
print(f'passed: {test_passes:<7}', end='')
print(f'total: {test_total:<9}', end='')
print(f'failed: {test_fails:<9}', end='')
print(f'aborted: {test_aborts:<10}')
print()
