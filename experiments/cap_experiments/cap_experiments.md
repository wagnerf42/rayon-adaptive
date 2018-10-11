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
The last run was with the vector free being moved out of the logging.
Following experiments were run after that:
* The cap was put at 100_000. Time was 1.9ms.
* The cap was put at 10_000. Time was 1.8ms.
* The cap was put at 1000. Time was 2.1ms.

The baseline for these experiments is no cap. Time was 2.1ms.
## Remarks
* Performance fluctuated slightly, cap 1000 performed worst and cap 10_000 seems to have performed the best.
## Conclusion
* Maybe the cap is too strict.
* Total time is too less, results could be disregarded as noise.
* Add display for total time spent per type of task.
* Increase the size of the vector.
* Make the number of threads prime so that splits become unpredictable.
* Increase the number of runs to 1000.
