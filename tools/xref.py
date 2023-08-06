import sys
from datetime import datetime

with open(sys.argv[1]) as f: xref = f.readlines()
date = datetime.now().strftime('%m/%d/%Y %H:%M:%S')

print(f'cross reference: {date:<10}')
print('----------------------')

line_no = 0
xref.sort()

for symbol in xref:
    fields = symbol[:-1].split("\t")
    name, type, value = fields
    if name == 'unbound':
        line_no += 1        
        print(f'{line_no:<5} {value:<35} {type:<10}')
print('----------------------')

line_no = 0
for symbol in xref:
    fields = symbol[:-1].split("\t")
    name, type, value = fields

    if name != 'unbound':
        line_no += 1
        fvalue = (value[:47] + '...') if len(value) > 50 else value
        print(f'{line_no:<5} {name:<35} {type:<10} {fvalue:<30}')
