#![no_main]

use coalesced_intervals::CoalescedIntervals;

use libfuzzer_sys::{arbitrary::{Arbitrary, Error, Unstructured}, fuzz_target};

#[derive(Debug)]
struct IntervalsToInsert {
    ivals: Vec<(i8, i8)>
}

impl<'a> Arbitrary<'a> for IntervalsToInsert {
    fn arbitrary(raw: &mut Unstructured<'a>) -> Result<Self, Error> {
        let size = raw.arbitrary_len::<u8>()?;
        let mut ivals = Vec::with_capacity(size);
        for _ in 0..size {
            let start: i8 = raw.int_in_range(-128..=127)?;
            let limit: i8 = raw.int_in_range(start..=127)?;
            ivals.push((start, limit));
        }

        Ok(IntervalsToInsert{ivals})
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
        coalesced.add((*start, *limit));
    }

    coalesced.check_invariants();

    let v = coalesced.to_vec();
    log::debug!("v: {:?}", v);
    check_vector(v)
});
