import json
import matplotlib.pyplot as plot
from collections import OrderedDict

import matplotlib.pyplot as plt

with open("../output.json", 'r') as file:
    data = json.load(file)
    cache_results = data["cache"]
    for test in cache_results.keys():
        cache_sizes = []
        times = []
        for tpl in cache_results[test]:
            key_str, value_str = next(iter(tpl.items()))
            cache_sizes.append(int(key_str))
            times.append(int(value_str))
        plot.plot(cache_sizes, times)
        plt.title(test)
        plot.xlabel("Cache Size")
        plot.ylabel("Time in µs")
        plot.show()
    print(cache_results)