import sys
from datetime import datetime

with open(sys.argv[1]) as f: xref = f.readlines()
date = datetime.now().strftime('%m/%d/%Y %H:%M:%S')

print(f'reverse cross reference: {date:<10}')
print('----------------------')

line_no = 0
revxref = []

for symbol in xref:
    fields = symbol[:-1].split("\t")
    name, type, value = fields

    if name != 'unbound' and type == 'func':
        tag = value[:-2].split()
        a, b, r, t = tag
        addr = t[4:].rjust(16, ' ')

        scope = 'extern'
        if name[0] == ':':
            name = name[1:]
            scope = 'intern'

        revxref.append((addr, type, scope, name))

revxref.sort()
for symbol in revxref:
    line_no += 1
    addr, type, scope, name = symbol;

    print(f'{line_no:<5} {addr:<16}    {type}  {scope}  {name:<30}')
