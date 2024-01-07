import json
import matplotlib.pyplot as plot

option_array = []

with open("../options.json", 'r') as file:
    data = json.load(file)
    options = data["options"]
    for option in options.items():
        print(option)
        option_array.append(option[0] + '=' + str(option[1]))

with open("../output.json", 'r') as file:
    data = json.load(file)
    cache_results = data["cachesize"]

    if cache_results:
        for test, sizes in cache_results.items():
            cache_sizes = []
            mean_times = []

            for size in sorted(sizes.keys(), key=int):
                time_values = sizes[size]
                mean_time = sum(int(time) for time in time_values) / len(time_values)
                cache_sizes.append(int(size))
                mean_times.append(mean_time)

            plot.figure()
            plot.plot(cache_sizes, mean_times, marker='o')
            plot.title(f"{test}-test {option_array}", fontsize=9)
            plot.xlabel("Cache Size")
            plot.ylabel("Mean Time per access in µs")
            plot.xticks(cache_sizes, cache_sizes)
            plot.grid(True)
            # plot.text(x=cache_sizes[-1], y=mean_times[-1], s="End Point", fontsize=10, verticalalignment='bottom')
            plot.savefig(f"../{test}-test_{option_array}.png", dpi=500)
            plot.show()

    num_accesses_results = data["numberofaccesses"]

    if num_accesses_results:
        for test, sizes in num_accesses_results.items():
            cache_sizes = []
            mean_times = []

            for size in sorted(sizes.keys(), key=int):
                time_values = sizes[size]
                mean_time = sum(int(time) for time in time_values) / len(time_values)
                cache_sizes.append(int(size))
                mean_times.append(mean_time)

            # plot.figure()
            plot.plot(cache_sizes, mean_times, marker='o')
            plot.title(f"{test}-test {option_array}", fontsize=9)
            plot.xlabel("Number of accesses")
            plot.ylabel("Total mean time in µs")
            plot.xticks(cache_sizes, cache_sizes)
            plot.grid(True)
            # plot.text(x=cache_sizes[-1], y=mean_times[-1], s="End Point", fontsize=10, verticalalignment='bottom')
        plot.savefig(f"../{test}-test_{option_array}.png", dpi=500)
        plot.show()