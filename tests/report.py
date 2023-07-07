import sys
from datetime import datetime

namespace=sys.argv[1]
test_name=sys.argv[2]

with open(sys.argv[3]) as f: test_results = f.readlines()
date = datetime.now().strftime('%m/%d/%Y %H:%M:%S')

print(f'Test Report: {namespace}/{test_name} {date:<10}')
print('----------------------')

line_no = 0
totals = [0, 0, 0, 0]
for test in test_results:
    line_no += 1

    fields = test[:-1].split("\t")
    test, expected, obtained, status = fields

    totals[0] += 1

    if status == "passed":
        totals[1] += 1
    elif status == "failed":
        totals[2] += 1
    elif status == "aborted":
        totals[3] += 1
    else:
        print("status: " + status)
        exit(1)

    if status == "aborted" or status == "failed":
        ftest = (test[:27] + '...') if len(test) > 30 else test
        ferr = (obtained[:27] + '...') if len(obtained) > 30 else obtained
        fexpect = (expected[:12] + '...') if len(expected) > 15 else expected
        print(f'{line_no:>3} {ftest:<30} {fexpect:<15} {ferr:<30} {status:<15}')

print('----------------------')
print(f'total tests: {totals[0]:<5} passed: {totals[1]:<5} failed: {totals[2]:<5} aborted: {totals[3]:<5}')    
