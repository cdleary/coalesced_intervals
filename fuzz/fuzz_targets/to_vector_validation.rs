#![no_main]

use std::collections::BTreeSet;

use coalesced_intervals::CoalescedIntervals;

use libfuzzer_sys::{arbitrary::{Arbitrary, Error, Unstructured}, fuzz_target};

#[derive(Debug)]
struct IntervalsToInsert {
    ivals: Vec<(i8, i8)>,
    /// Start values in non-zero-sized segments.
    true_starts: BTreeSet<i8>,

    included: BTreeSet<i8>,
}

impl<'a> Arbitrary<'a> for IntervalsToInsert {
    fn arbitrary(raw: &mut Unstructured<'a>) -> Result<Self, Error> {
        let mut included = BTreeSet::new();
        let mut true_starts = BTreeSet::new();
        let size = raw.arbitrary_len::<u8>()?;
        let mut ivals = Vec::with_capacity(size);
        for _ in 0..size {
            let start: i8 = raw.int_in_range(-128..=127)?;
            let limit: i8 = raw.int_in_range(start..=127)?;
            if start != limit {
                true_starts.insert(start);
            }
            ivals.push((start, limit));
            for i in start..limit {
                included.insert(i);
            }
        }

        Ok(IntervalsToInsert{ivals, true_starts, included})
    }
}

fn check_vector(v: Vec<(i8, i8)>) {
    // Check the vector is sorted.
    let mut v_clone = v.clone();
    v_clone.sort();
    assert_eq!(v, v_clone);

    for i in 0..v.len() {
        let (start, limit) = v[i];
        assert!(limit > start);
        if i > 0 {
            let (_prev_start, prev_limit) = v[i-1];
            // This start should be > the prev limit or they should have been coalesced.
            assert!(start > prev_limit);
        }
        if i+1 < v.len() {
            let (next_start, _next_limit) = v[i+1];
            // This limit should be < the prev start or they should have been coalesced.
            assert!(limit < next_start);
        }
    }
}

fuzz_target!(|ivals: IntervalsToInsert| {
    let mut coalesced = CoalescedIntervals::<i8>::new();
    for (start, limit) in ivals.ivals.iter() {
        coalesced.add(*start, *limit);
    }

    coalesced.check_invariants();

    // Check the "get_interval_containing" result is appropriately some/none for every value we
    // determined is included in the (small) range under test.
    for i in i8::MIN..i8::MAX {
        if ivals.included.contains(&i) {
            assert!(coalesced.get_interval_containing(i).is_some());
        } else {
            assert!(coalesced.get_interval_containing(i).is_none());
        }
    }

    // Traverse all the intervals using the "get_first_start_from" function to check its
    // functionality.
    let mut find_start_from = i8::MIN;
    let mut last_limit_seen = i8::MIN;

    let mut seen = vec![];
    let max_start = ivals.true_starts.iter().max();
    loop {
        match coalesced.get_first_start_from(find_start_from) {
            None => {
                match max_start {
                    None => {},
                    Some(max_start_value) => {
                        assert!(last_limit_seen > *max_start_value, "Attempted to find start from {} but it turned up nothing; supposedly max start value is {}; last limit seen is {}", find_start_from, *max_start_value, last_limit_seen);
                    },
                }
                break;
            },
            Some((ival_start, ival_limit)) => {
                find_start_from = ival_start + 1;
                last_limit_seen = ival_limit;
                seen.push((ival_start, ival_limit));
            }
        }
    }

    let v = coalesced.to_vec();

    // All of the sorted pairs should have been observed in that same order in the "first start
    // from" based traversal.
    assert_eq!(v, seen);

    log::debug!("v: {:?}", v);
    check_vector(v)
});
