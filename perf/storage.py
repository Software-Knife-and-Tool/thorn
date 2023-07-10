import sys
from statistics import mean
from datetime import datetime

namespaces = [
    'mu:',
    'core:',
]

with open(sys.argv[1]) as f: test_results = f.readlines()
date = datetime.now().strftime('%m/%d/%Y %H:%M:%S')

print(f'Storage Report: {date:<10}')
print('----------------------')

types = [1, 5, 9, 13, 17, 21, 25]
def storage(test_name, hp_info):
    fields = hp_info[:-1].split()
    if len(fields) == 29:
        total = 0
        for type in types:
            total += int(fields[type + 1])
            
        print(f'{test_name:<16} {total:<5}', end='')
        for type in types:
            name = fields[type]
            total = int(fields[type + 1])
            alloc = int(fields[type + 2])
            in_use = int(fields[type + 3])

            if total != 0:
                print(f' {name:<5} ({total} {alloc} {in_use})', end='')
        print()

                
for namespace in namespaces:
    for test in test_results:
        fields = test[:-1].split(',')
        if fields[0] == namespace:
            if len(fields) > 2:
                storage(fields[1], fields[2])

print('----------------------')
