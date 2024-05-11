# `coalesced_intervals`: maintain maximally coalesced 1D intervals

```
extern crate coalesced_intervals;

fn main() {
    let mut ivals = coalesced_intervals::CoalescedIntervals::new();
    ivals.add((0, 1));
    ivals.add((2, 3));
    assert_eq!(ivals.to_vec(), [(0, 1), (2, 3)]);

    ivals.add((1, 2));
    assert_eq!(ivals.to_vec(), [(0, 3)]);
}
```
