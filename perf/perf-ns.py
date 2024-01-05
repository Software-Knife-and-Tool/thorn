###  SPDX-FileCopyrightText: Copyright 2023 James M. Putnam (putnamjm.design@gmail.com)
###  SPDX-License-Identifier: MIT
###
import json
import os
import sys
import subprocess

ns = sys.argv[1]
ns_path = sys.argv[2]
ntests = sys.argv[3]

with open(os.path.join(ns_path, ns, 'tests')) as f: perf_groups = f.readlines()

def storage(ns, group, line, test):
    if ns == 'mu':
        proc = subprocess.Popen(['../dist/mu-shell',
                                 '-p',
                                 '-l./perf.l',
                                 '-e (mu:%sdelta (:lambda ()' + test + ') :nil)'],\
                                stdout=subprocess.PIPE,\
                                stderr=subprocess.PIPE)

    if ns == 'frequent':
        proc = subprocess.Popen(['../dist/mu-shell',
                                 '-p',
                                 '-l./perf.l',
                                 '-e (mu:%sdelta (:lambda ()' + test + ') :nil)'],\
                                stdout=subprocess.PIPE,\
                                stderr=subprocess.PIPE)

    if ns == 'prelude':
        proc = subprocess.Popen(['../dist/mu-shell',
                                 '-l../dist/prelude.l',
                                 '-q (prelude:%init-ns)',
                                 '-p',
                                 '-l./perf.l',
                                 '-e (mu:%sdelta (:lambda ()' + test + ') :nil)'],\
                                stdout=subprocess.PIPE,\
                                stderr=subprocess.PIPE)
    
    storage_ = proc.stdout.read()[:-1].decode('utf8')
    err = proc.stderr.read()[:-1].decode('utf8')

    proc.communicate()

    if len(err) != 0:
        print(f'exception: {ns}/{group}:{line:<5} {err}', file=sys.stderr)
    
    return None if len(err) != 0 else storage_

def timing(ns, test):
    if ns == 'mu':
        proc = subprocess.Popen(['../dist/mu-shell',
                                 '-p',
                                 '-l./perf.l',
                                 '-e (mu:%tdelta (:lambda ()' + test + ') :nil)'],\
                                stdout=subprocess.PIPE,\
                                stderr=subprocess.PIPE)

    if ns == 'frequent':
        proc = subprocess.Popen(['../dist/mu-shell',
                                 '-p',
                                 '-l./perf.l',
                                 '-e (mu:%tdelta (:lambda ()' + test + ') :nil)'],\
                                stdout=subprocess.PIPE,\
                                stderr=subprocess.PIPE)

    if ns == 'prelude':
        proc = subprocess.Popen(['../dist/mu-shell',
                                 '-l../dist/prelude.l',
                                 '-q (prelude:%init-ns)',
                                 '-p',
                                 '-l./perf.l',
                                 '-e (mu:%tdelta (:lambda ()' + test + ') :nil)'],\
                                stdout=subprocess.PIPE,\
                                stderr=subprocess.PIPE)
    
    time = proc.stdout.read()[:-1].decode('utf8')
    err = proc.stderr.read()[:-1].decode('utf8')

    proc.communicate()

    return None if proc.poll == 0 else time

ns_results = []
for group in perf_groups:
    with open(os.path.join(ns_path, ns, group[:-1])) as f: group_source = f.readlines()

    storage_ = None
    results = []

    line = 0
    for test in group_source:
        line += 1
        storage_ = storage(ns, group[:-1], line, test[:-1])

        if storage_ == None:
            break

        times = []
        for n in range(int(ntests)):
            times.append(timing(ns, test[:-1]))

        results.append({ 'line': line, 'storage': storage_, 'times': times })

    ns_results.append({'group': group[:-1], 'results': results })

print(json.dumps({ 'ns': sys.argv[1], 'results': ns_results }))
