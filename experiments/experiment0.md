## Experimental setup
* Threads = 8. <br>
* Benchmark without logging, binary with logging.<br>
* Random vector setup in all runs of benchmark and bin. Time for vector generation is not included. <br>
* Vector size = 4 million. <br>
* Policy block size = 1000. <br>
* Running on dahu-8 for log.
* Running on dahu-15 for benchmark.
* Rayon v1.02
* Rayon-Logs commit = 11be3efdcc96843a46462a42a0706b296cb093ad
* Rayon-Adaptive commit = 58e2dba8509b75eafb82590cf0639cecd575b421
## Remarks
* Rayon par split code has significant time difference in the benchmark and the logs
* What is the big horizontal bar of task in the end?
## TODO
1. Reproduce the results with full control of one node.
2. Run bin/infix.rs with no logging 100 times and average out the result. See if this matches with the benchmark?

