# Benchmark Findings

Simple notes from benchmarks that have been run

## Collection types

Four collections were compared; `Vec` (as a baseline),
`std::collections::BTreeSet`, `std::collections::HashSet`, and
`hashbrown::HashSet`.  These were each tested on `.contains()` with values that
did and did not exist, as well as

```
               Vec          BTreeSet     std HashSet    hashbrown HashSet
contains       594  us      2.17 us      530 ns         279 ns
not contains   1.91 us      2.40 us      436 ns         160 ns
collect        18.3 us      301  us      204 us         120 us
```

The `HashSet` implementations significantly beat out other alternatives, and the
`hashbrown` implementation outperformed `std::HashSet`. This is expected because
`hashbrown` uses a faster hash that is not cryptographically secure (not a
problem for our applications).

For some reason, the improvements going from `std` to `hashbrown` don't really
seem to show up for the dictionary integration tests. This will take some
looking into.

## Slice `contains` vs. `binary_search`

Overall, the price of sorting doesn't seem to have any payoff, especially for
our use cases of short arrays. If it is already sorted then we can save time,
about 20% on average.
