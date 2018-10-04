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
## Conclusion
1. The big single task at the end is a memory related system call for dropping the vector which is huge and on the heap. It is being shown like that because there is some task being tagged before this and that is leading to the creation of a separate logged task just for the drop of the memory. Furthermore, this was not visible earlier because the vector was not getting generated each time (it was shared accross threads and not passed in the closure).
2. The time for par_split is much more in the case of the logs because the distribution inherently is very variable. The way Criterion is sampling and averaging differs from the way we are. Furthermore, it is fundamentally drawing orders of magnitude more samples than us, so that will also have an effect.
