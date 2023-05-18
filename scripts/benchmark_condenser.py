import sys
import cbor2
import os
from os import path

def get_meta_data(dir: str):
    meta_path = path.join(dir, "benchmark.cbor")
    print(f"Reading meta file `{meta_path}`")
    with open(meta_path, 'rb') as file:
        return cbor2.load(file)

def get_criterion_data(dir: str):
    benchmark_meta = get_meta_data(dir)
    
    benchmark_path = path.join(dir, benchmark_meta["latest_record"])
    print(f"Reading bench file `{benchmark_path}`")
    with open(benchmark_path, 'rb') as file:
        data = cbor2.load(file)

    thrpt = benchmark_meta["id"]["throughput"]
    upper = data["estimates"]["mean"]["confidence_interval"]["upper_bound"]

    return (thrpt["Elements"], upper) if thrpt else (None, upper) 

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print('Must provide a base directory.')
        exit(1)

    base_directory = path.abspath(sys.argv[1])

    write_directory = path.abspath(sys.argv[2] if len(sys.argv) > 2 else ".")

    overview_table_file = path.join(write_directory, "table.csv")
    table_file = open(overview_table_file, "w")
    table_file.write("# regex, ns (compiled), ns (interpreted)\n")

    passed_dirs = set()

    # estimates.mean.confidence_interval.upper_bound
    for dir in os.listdir(base_directory):
        dir_path = path.join(base_directory, dir)
        if not path.isdir(dir_path) or dir in passed_dirs:
            continue
        else:
            print(f"Reading `{dir_path}`")
            passed_dirs.add(dir)

            if "continuous" in dir:
                subdirs = os.listdir(path.join(base_directory, dir_path))
                subdirs.sort(key=int)
                dir_items = map(lambda x: path.join(base_directory, dir_path, x), subdirs)
                data_items = map(get_criterion_data, dir_items)

                file_name = path.join(write_directory, dir + ".csv")
                file_name.replace(" ", "-")

                with open(file_name, "w") as file:
                    file.write("# n, ns\n")
                    for (n, time) in data_items:
                        file.write(f"{n}, {time}\n")
            else:
                if "compiled" in dir:
                    compiled_path = dir_path
                    interpreted_path = path.join(base_directory, dir.replace("compiled", "interpreted"))

                    passed_dirs.add(dir.replace("compiled", "interpreted"))
                else:
                    compiled_path = dir_path.replace("interpreted", "compiled")
                    interpreted_path = dir_path

                    passed_dirs.add(dir.replace("interpreted", "compiled"))

                if len(os.listdir(compiled_path)) < 2:
                    number = os.listdir(compiled_path).pop()
                    compiled_path = path.join(compiled_path, number)
                    interpreted_path = path.join(interpreted_path, number)

                    print(f"Diving to `{compiled_path}`")
                    print(f"Diving to `{interpreted_path}`")


                (_, compiled_data) = get_criterion_data(compiled_path)
                (_, interpreted_data) = get_criterion_data(interpreted_path)


                regex_name = get_meta_data(compiled_path)["id"]["group_id"].split(" ").pop()

                table_file.write(f"{regex_name}, {compiled_data}, {interpreted_data}\n")
        print()

    table_file.flush()
    table_file.close()
