# `coalesced_intervals`: maintain maximally coalesced 1D intervals

![sample usage diagram](https://raw.githubusercontent.com/cdleary/coalesced_intervals/main/docs/coalesced-intervals.png)

```rust
extern crate coalesced_intervals;

fn main() {
    let mut ivals = coalesced_intervals::CoalescedIntervals::new();

    // Add `[0, 1)` and `[2, 3)` (there's a hole in the middle).
    ivals.add(0, 1);
    ivals.add(2, 3);
    assert_eq!(ivals.to_vec(), [(0, 1), (2, 3)]);

    // By adding `[1, 2)` we end up with a coalesced segment `[0, 3)`.
    ivals.add(1, 2);
    assert_eq!(ivals.to_vec(), [(0, 3)]);

    // We can ask for the interval containing some target value.
    assert_eq!(ivals.get_interval_containing(1), Some((0, 3)));
    assert_eq!(ivals.get_interval_containing(4), None);

    // We can ask for the interval that starts at-or-after some value.
    assert_eq!(ivals.get_first_start_from(-1), Some((0, 3)));
    assert_eq!(ivals.get_first_start_from(0), Some((0, 3)));
    assert_eq!(ivals.get_first_start_from(1), None);
}
```
