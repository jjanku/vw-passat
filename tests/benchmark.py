import argparse
import csv
import os
import os.path as path
import re
import subprocess
import time


# paths are relative to this file
DATA = './data'
SOLVER = '../target/release/vw-passat'


def run_solver(input, timeout = 10, jobs = None):
    args = [SOLVER]
    if jobs is not None:
        args.extend(['-j', jobs])
    args.append(input)

    subprocess.run(
        args, timeout=timeout, capture_output=True
    )


def benchmark(dir, jobs):
    name = path.basename(dir)
    files = os.listdir(dir)

    total = len(files)
    solved = 0

    def print_progress(end=''):
        print(f'\r{name:16} {solved:4} / {total:4}', end=end)

    results = []

    for file in files:
        print_progress()
        try:
            start = time.perf_counter()
            run_solver(path.join(dir, file), jobs=jobs)
            end = time.perf_counter()
            duration = end - start
            results.append((file, duration))
            solved += 1
        except subprocess.TimeoutExpired:
            results.append((file, None))
    print_progress(end='\n')

    return results


def dir_key(dir):
    unsat = dir.startswith('uu')
    var_count = int(dir.strip('uf').split('-')[0])
    return var_count, unsat


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        '--filter', help='run tests only in dirs matching FILTER'
    )
    parser.add_argument(
        '-j', '--jobs', help='run vw-passat with -j JOBS'
    )
    parser.add_argument(
        '-o', '--output', help='generate a CSV file with the results'
    )
    args = parser.parse_args()

    dir_pattern = re.compile(
        args.filter if args.filter is not None
        else '.'
    )

    os.chdir(path.dirname(__file__))

    dirs = os.listdir(DATA)
    dirs.sort(key=dir_key)

    total = 0
    solved = 0

    if args.output is not None:
        writer = csv.writer(open(args.output, 'w', newline=''))
        writer.writerow(['file', 'time'])
    else:
        writer = None

    for dir in dirs:
        if dir_pattern.search(dir) is None:
            continue
        try:
            results = benchmark(path.join(DATA, dir), args.jobs)
            if writer is not None:
                writer.writerows(results)
            solved += sum(t is not None for _, t in results)
            total += len(results)
        except KeyboardInterrupt:
            print(' (Skipped)')

    print(f'Summary: {solved:4} / {total:4}')


if __name__ == '__main__':
    main()
