import sys
from statistics import mean
from datetime import datetime

namespaces = [
    'mu:',
    'core:',
]

with open(sys.argv[1]) as f: test_results = f.readlines()
date = datetime.now().strftime('%m/%d/%Y %H:%M:%S')

totals = [2, 6, 10, 14, 18, 22, 26]
def storage_bytes(hp_info):
    fields = hp_info[:-1].split()
    if len(fields) < 29:
        return -1
    total = 0
    for i in totals:
        total += int(fields[i])
    return total

def time_average(times):
    if len(times) == 0:
        return -1
    if not times[0].isdigit():
        return -1
    return mean(list(map(int, times)))

for namespace in namespaces:
    for test in test_results:
        fields = test[:-1].split(',')
        if fields[0] == namespace:
            if len(fields) > 2:
                test_name = fields[1]
                total_bytes = storage_bytes(fields[2])
                avg_usec = time_average(fields[3:])
                if avg_usec > 0:
                    print(f'{test_name:<16} {total_bytes:<5} {avg_usec:.2f}')
