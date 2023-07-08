import sys
# from statistics import mean

with open(sys.argv[1]) as f: test_results = f.readlines()

test_name = ""

total_storage = 0
total_time = 0.0

for test in test_results:
    name, storage, time = test[:-1].split()

    if name == test_name:
        total_storage += int(storage)
        total_time += float(time)
    else:
        if test_name != "":
            print(f"{test_name:<15} {total_storage:>8} {total_time:15.2f}")
        test_name = name
        total_storage = 0
        total_time = 0.0
