import json
import sys

from statistics import mean

with open(sys.argv[1]) as f: test_results = json.load(f)
ns = test_results['ns']

# [2, 6, 10, 14, 18, 22, 26]
def storage_bytes(hp_info):
    fields = hp_info[:-1].split()

    if len(fields) != 25:
        return 0

    total = 0
    for i in range(2, 26, 4):
        total += int(fields[i])
    return total

def time_average(times):
    return mean(list(map(int, times)))

results = []
for group in test_results['results']:
    for test in group['results']:
        if test['storage'] == '':
            results.append({ 'test': ns + '/' + group['group'],
                             'line': test['line'],
                             'storage': 0,
                             'times': 0.0 })
        else:
            results.append({ 'test': ns + '/' + group['group'],
                             'line': test['line'],
                             'storage': storage_bytes(test['storage']),
                             'times': time_average(test['times']) })

for test in results:
    test_name, line, storage, times = test.values()
    print(f'{line:>02d} {test_name:<18} {storage:>6} {times:8.2f}')
