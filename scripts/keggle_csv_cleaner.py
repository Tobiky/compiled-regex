import sys
import pathlib
import csv

if __name__ == '__main__':
    file_path = sys.argv[1]

    if not pathlib.Path(file_path).exists():
        print("Please provide a file that exists.")
        exit(1)

    with open(file_path) as file:
        reader = csv.reader(file)

        for row in reader:
            print(row[0])
