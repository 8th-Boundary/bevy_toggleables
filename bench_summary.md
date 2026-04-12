Updated with the larger runs too.

## 100 entities

| Family      |    table |  sparse | entity map | toggle table | toggle sparse | Fastest      |
| ----------- | -------: | ------: | ---------: | -----------: | ------------: | ------------ |
| Add         |  6.50 us | 4.99 us |    1.61 us |     1.26 us* |      1.27 us* | toggle table |
| Modify      |  1.25 us | 1.30 us |    2.14 us |      1.28 us |       1.29 us | table        |
| Remove      |  6.18 us | 5.27 us |    2.59 us |     1.25 us* |      1.32 us* | toggle table |
| Rapid cycle | 10.63 us | 9.72 us |    3.36 us |     1.30 us* |      1.35 us* | toggle table |

## 5000 entities

| Family      |     table |    sparse | entity map | toggle table | toggle sparse | Fastest      |
| ----------- | --------: | --------: | ---------: | -----------: | ------------: | ------------ |
| Add         | 152.79 us | 129.02 us |   53.63 us |     7.70 us* |     10.22 us* | toggle table |
| Modify      |   6.97 us |  10.19 us |   98.79 us |      7.46 us |       9.51 us | table        |
| Remove      | 223.90 us | 195.50 us |  104.68 us |     8.62 us* |     11.63 us* | toggle table |
| Rapid cycle | 339.83 us | 316.10 us |  139.84 us |     9.22 us* |     12.58 us* | toggle table |

## 25000 entities

| Family      |     table |    sparse | entity map | toggle table | toggle sparse | Fastest      |
| ----------- | --------: | --------: | ---------: | -----------: | ------------: | ------------ |
| Add         | 1.0159 ms | 654.21 us |  311.99 us |    31.11 us* |     51.35 us* | toggle table |
| Modify      |  24.14 us |  36.13 us |  570.48 us |     27.69 us |      41.77 us | table        |
| Remove      | 1.0832 ms | 978.08 us |  601.08 us |    26.10 us* |     40.49 us* | toggle table |
| Rapid cycle | 1.6841 ms | 1.5303 ms |  711.44 us |    39.03 us* |     47.18 us* | toggle table |

## 50000 entities

| Family      |     table |    sparse | entity map | toggle table | toggle sparse | Fastest      |
| ----------- | --------: | --------: | ---------: | -----------: | ------------: | ------------ |
| Add         | 1.9760 ms | 1.6675 ms |  904.31 us |   133.03 us* |    163.01 us* | toggle table |
| Modify      | 113.11 us | 160.99 us |  1.6135 ms |    121.48 us |     164.03 us | table        |
| Remove      | 2.4769 ms | 2.2355 ms |  1.5526 ms |   120.95 us* |    164.29 us* | toggle table |
| Rapid cycle | 3.5762 ms | 3.2492 ms |  2.0271 ms |   135.01 us* |    172.20 us* | toggle table |

## 100000 entities

| Family      |     table |    sparse | entity map | toggle table | toggle sparse | Fastest      |
| ----------- | --------: | --------: | ---------: | -----------: | ------------: | ------------ |
| Add         | 4.0431 ms | 3.9616 ms |  1.9014 ms |   301.81 us* |    391.82 us* | toggle table |
| Modify      | 229.73 us | 353.30 us |  3.0461 ms |    269.17 us |     338.09 us | table        |
| Remove      | 5.1445 ms | 4.7111 ms |  3.1309 ms |   284.32 us* |    347.49 us* | toggle table |
| Rapid cycle | 7.0568 ms | 6.5716 ms |  4.4371 ms |   306.18 us* |    346.96 us* | toggle table |

## 500000 entities

| Family      |     table |    sparse | entity map | toggle table | toggle sparse | Fastest      |
| ----------- | --------: | --------: | ---------: | -----------: | ------------: | ------------ |
| Add         | 18.507 ms | 17.660 ms |  12.459 ms |   1.2844 ms* |    2.3796 ms* | toggle table |
| Modify      | 1.2910 ms | 1.7210 ms |  44.273 ms |    1.5018 ms |     1.9655 ms | table        |
| Remove      | 29.459 ms | 25.497 ms |  48.288 ms |   1.4344 ms* |    2.0741 ms* | toggle table |
| Rapid cycle | 42.619 ms | 36.993 ms |  44.128 ms |   1.6978 ms* |    2.0071 ms* | toggle table |

* Toggle columns use the **logical** path (`enable_disabled`, `disable_logical`, `cycle_logical`) rather than the structural toggle path.

The trend is very clean now: **table** keeps winning pure updates, while **toggle table** keeps winning every churn-heavy family, even at very large scales.
