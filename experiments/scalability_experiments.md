## Objective
To find out what is the input size required for scalability and also compare how the algorithms compare on this front.

## Experiments
Experiments run with 7 threads:
0. Size 1 million
1. Size 10 million
2. Size 50 million
3. Size 100 million

Experiments run with 8 threads:
4. Size 1 million
5. Size 10 million
6. Size 50 million
7. Size 100 million

Experiments run with 13 threads:
8. Size 1 million
9. Size 10 million
10. Size 50 million
11. Size 100 million

Experiments run with 16 threads:
12. Size 1 million
13. Size 10 million
14. Size 50 million
15. Size 100 million


## Results

### Adaptive scaling:
<table>
<tr><td>Size in millions</td><td>Threads</td><td>Speedup</td></tr>
<tr><td>1</td><td>7</td><td>2.06</td></tr>
<tr><td>1</td><td>8</td><td>2.39</td></tr>
<tr><td>1</td><td>13</td><td>2.55</td></tr>
<tr><td>1</td><td>16</td><td>2.64</td></tr>
<tr><td>10</td><td>7</td><td>3.17</td></tr>
<tr><td>10</td><td>8</td><td>3.47</td></tr>
<tr><td>10</td><td>13</td><td>5.50</td></tr>
<tr><td>10</td><td>16</td><td>9.01</td></tr>
<tr><td>50</td><td>7</td><td>2.85</td></tr>
<tr><td>50</td><td>8</td><td>3.61</td></tr>
<tr><td>50</td><td>13</td><td>2.74 (Redo)</td></tr>
<tr><td>50</td><td>16</td><td>3.77 </td></tr>
<tr><td>100</td><td>7</td><td>2.96</td></tr>
<tr><td>100</td><td>8</td><td>3.04</td></tr>
<tr><td>100</td><td>13</td><td>4.27</td></tr>
<tr><td>100</td><td>16</td><td>6.55</td></tr>
</table>

## Questions

- why does speedup decrease when sizes increase ?
- we don't see min input size for which we have speedup
- we don't see low number of threads : how many cores needed to be efficient ?
- show me a cloud of points : then we can see stability
- show me curves : to exhibit behaviours
- maybe you need less than 500 runs ?
- many algorithms, only one presented
