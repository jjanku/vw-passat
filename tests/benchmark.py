import argparse
import os
import os.path as path
import re
import subprocess


# paths are relative to this file
DATA = './data'
SOLVER = '../target/release/vw-passat'


def run_solver(input, timeout = 10):
    subprocess.run(
        [SOLVER, input], timeout=timeout, capture_output=True
    )


def benchmark(dir):
    name = path.basename(dir)
    files = os.listdir(dir)

    total = len(files)
    solved = 0

    def print_progress(end=''):
        print(f'\r{name:16} {solved:4} / {total:4}', end=end)

    for file in files:
        print_progress()
        try:
            run_solver(path.join(dir, file))
            solved += 1
        except subprocess.TimeoutExpired:
            pass
    print_progress(end='\n')

    return solved, total


def dir_key(dir):
    unsat = dir.startswith('uu')
    var_count = int(dir.strip('uf').split('-')[0])
    return var_count, unsat


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        '--filter', help='run tests only in dirs matching FILTER'
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

    for dir in dirs:
        if dir_pattern.search(dir) is None:
            continue
        try:
            res = benchmark(path.join(DATA, dir))
            solved += res[0]
            total += res[1]
        except KeyboardInterrupt:
            print(' (Skipped)')

    print(f'Summary: {solved:4} / {total:4}')


if __name__ == '__main__':
    main()
