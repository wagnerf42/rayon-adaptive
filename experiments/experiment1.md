## Experimental setup
* Threads = 8. <br>
* Benchmark without logging, binary with logging.<br>
* Random vector setup in all runs of benchmark and bin. Time for vector generation is not included. <br>
* Vector size = 4 million. <br>
* Policy block size = 1000. <br>
* Running on dahu-9 for log.
* Running on dahu-9 for benchmark.
* Rayon v1.02
* Rayon-Logs commit = 11be3efdcc96843a46462a42a0706b296cb093ad
* Rayon-Adaptive commit = a86a53593d41a85784d7062a399e04155225fe9b
## Changelog
Following things have changed since experiment 0:
* Vector dropping has been moved out from the logging context by returning the vector from the closure.
* Small vec has been used to speed up sequential processing.
* A cap has been added on the block size (100_000)  for adaptive scheduling.
## Expectations
* This should speed up the rayon-adaptive.
## Remarks
* Performance improved by about 2 ms for all.
* Slight difference in benchmark vs logs. Seems to be either ways and hence should not be significant.
## Conclusion
* Cap does not seem to have had any effect.
* Should try with a stricter cap.

