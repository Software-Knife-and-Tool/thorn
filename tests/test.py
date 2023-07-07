import sys

ns_name = sys.argv[1]
test_name = sys.argv[2]

with open(sys.argv[3]) as f: test_results = f.readlines()

totals = [0, 0, 0, 0]
for test in test_results:
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
        
print(f'{ns_name},{test_name},{totals[0]},{totals[1]},{totals[2]},{totals[3]}')    
