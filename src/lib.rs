use core::ops::Bound;
/// This is a conceptually simple data structure designed for the case where you have intervals
/// that you'd like to coalesce into maximal contiguous runs.
///
/// For example, if I add `[0, 1)` and then I add `[1, 2)` I should observe a single contiguous
/// interval `[0, 2)` in the data structure.
use std::collections::BTreeMap;

/// Implementation note: we use two btrees, one with the starts as the keys and one with limits as
/// the keys.
pub struct CoalescedIntervals<T> {
    start_to_limit: BTreeMap<T, T>,
    limit_to_start: BTreeMap<T, T>,
}

impl<T: Copy + std::cmp::Ord + std::fmt::Debug> CoalescedIntervals<T> {
    pub fn new() -> Self {
        CoalescedIntervals {
            start_to_limit: BTreeMap::new(),
            limit_to_start: BTreeMap::new(),
        }
    }

    pub fn check_invariants(&self) {
        // There should be no empty-sized intervals, and data should be reflected symmetrically in
        // both maps.
        for (start, limit) in self.start_to_limit.iter() {
            assert!(start != limit);
            assert!(self.limit_to_start[&limit] == *start);
        }
        for (limit, start) in self.limit_to_start.iter() {
            assert!(start != limit);
            assert!(self.start_to_limit[&start] == *limit);
        }
    }

    /// To be dominated by this interval the candidate_start must be >= start and candidate_limit
    /// must be <= limit.
    fn remove_intervals_dominated_by(&mut self, start: T, limit: T) {
        let mut dominated = vec![];
        for (candidate_start, candidate_limit) in self
            .start_to_limit
            .range((Bound::Included(start), Bound::Excluded(limit)))
        {
            if *candidate_limit <= limit {
                dominated.push((*candidate_start, *candidate_limit));
            } else {
                // candidate_limit > limit, so we can stop looking
                break;
            }
        }
        for (s, l) in dominated {
            self.start_to_limit.remove(&s);
            self.limit_to_start.remove(&l);
        }
    }

    fn is_dominated_by_existing(&self, start: T, limit: T) -> bool {
        // Look at the first segment that ends at-or-after limit to see if it dominates.
        for (_existing_limit, existing_start) in self
            .limit_to_start
            .range((Bound::Included(limit), Bound::Unbounded))
        {
            if *existing_start <= start {
                return true;
            } else {
                break;
            }
        }
        // Look at the first segment that start at-or-before start to see if it dominates.
        for (_existing_start, existing_limit) in self
            .start_to_limit
            .range((Bound::Unbounded, Bound::Included(start)))
        {
            if *existing_limit >= limit {
                return true;
            } else {
                break;
            }
        }
        return false;
    }

    fn insert_record(&mut self, start: T, limit: T) {
        log::debug!("inserting record: {:?}, {:?}", start, limit);
        self.start_to_limit.insert(start, limit);
        self.limit_to_start.insert(limit, start);
    }

    fn remove_with_start_at(&mut self, value: T) -> T {
        if let Some((start, limit)) = self.start_to_limit.remove_entry(&value) {
            self.limit_to_start.remove(&limit);
            log::debug!("removed: {:?}, {:?}", start, limit);
            limit
        } else {
            panic!("Attempted to remove start that was not present in map");
        }
    }

    fn remove_with_limit_at(&mut self, value: T) -> T {
        if let Some((limit, start)) = self.limit_to_start.remove_entry(&value) {
            self.start_to_limit.remove(&start);
            log::debug!("removed: {:?}, {:?}", start, limit);
            start
        } else {
            panic!("Attempted to remove limit that was not present in map");
        }
    }

    /// Finds any collision with the left edge of the segment; e.g. where the limit of another
    /// segment is contained within this segment; i.e.
    ///
    /// `start <= other.limit <= limit`
    fn find_collision_left(&self, start: T, limit: T) -> Option<(T, T)> {
        for (other_limit, other_start) in self
            .limit_to_start
            .range((Bound::Included(start), Bound::Included(limit)))
        {
            if start <= *other_limit && *other_limit <= limit {
                return Some((*other_start, *other_limit));
            }
        }
        return None;
    }

    /// Finds any collision with the right edge of the segment; e.g. where the start of another
    /// segment is contained within this segment; i.e.
    ///
    /// `start <= other.start <= limit`
    fn find_collision_right(&self, start: T, limit: T) -> Option<(T, T)> {
        for (other_start, other_limit) in self
            .start_to_limit
            .range((Bound::Included(start), Bound::Included(limit)))
        {
            if start <= *other_start && *other_start <= limit {
                return Some((*other_start, *other_limit));
            }
        }
        return None;
    }

    pub fn add(&mut self, item: (T, T)) {
        let (start, limit) = item;

        // Ignore empty segments.
        if start == limit {
            return;
        }

        // No change necessary if there's already an interval in there that dominates this one.
        if self.is_dominated_by_existing(start, limit) {
            return;
        }

        self.remove_intervals_dominated_by(start, limit);

        // If our start is another segment's limit, or our limit is another segment's start, we
        // coalesce them. Note that both may be true simultaneously. The invariant is that
        // we're maximally coalesced as an invariant, so we don' thave to look for additional
        // things that coalesce or are dominated by this new larger block, they would have been
        // colliding which would break the invariant.

        let collision_left: Option<(T, T)> = self.find_collision_left(start, limit);
        let collision_right: Option<(T, T)> = self.find_collision_right(start, limit);

        log::debug!("considering: {:?}, {:?}", start, limit);
        log::debug!("  collision left:  {:?}", collision_left);
        log::debug!("  collision right: {:?}", collision_right);

        match (collision_left, collision_right) {
            (None, None) => {
                self.insert_record(start, limit);
            }
            // Collision on the right edge.
            (None, Some((collided_start, collided_limit))) => {
                self.remove_with_start_at(collided_start);
                assert!(collided_limit > limit);
                self.insert_record(start, collided_limit);
            }
            // Collision on the left edge.
            (Some((collided_start, _collided_limit)), None) => {
                self.remove_with_start_at(collided_start);
                assert!(collided_start < start);
                self.insert_record(collided_start, limit);
            }
            // Collision on both edges.
            (Some((left_start, _)), Some((_, right_limit))) => {
                self.remove_with_start_at(left_start);
                self.remove_with_limit_at(right_limit);
                assert!(left_start < start);
                assert!(limit < right_limit);
                self.insert_record(left_start, right_limit);
            }
        }
    }

    pub fn to_vec(&self) -> Vec<(T, T)> {
        let mut v: Vec<(T, T)> = Vec::with_capacity(self.start_to_limit.len());
        for (start, limit) in self.start_to_limit.iter() {
            v.push((*start, *limit));
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let ivals = CoalescedIntervals::<i64>::new();
        assert_eq!(ivals.to_vec(), []);
    }

    /// Adding a single segment with no area.
    #[test]
    fn with_empty_range() {
        let mut ivals = CoalescedIntervals::<i64>::new();
        ivals.add((0, 0));
        assert_eq!(ivals.to_vec(), []);
    }

    /// Adding a single segment (that has area in it).
    #[test]
    fn one_segment_range() {
        let mut ivals = CoalescedIntervals::<i64>::new();
        ivals.add((0, 1));
        assert_eq!(ivals.to_vec(), [(0, 1)]);
    }

    /// Adding two segments that coalesce.
    #[test]
    fn two_segment_abutted() {
        let mut ivals = CoalescedIntervals::<i64>::new();
        ivals.add((0, 1));
        ivals.add((1, 2));
        assert_eq!(ivals.to_vec(), [(0, 2)]);
    }

    /// Adding three segments that coalesce when third one shows up.
    #[test]
    fn three_segment_abutted() {
        let _ = env_logger::try_init();
        let mut ivals = CoalescedIntervals::<i64>::new();
        ivals.add((0, 1));
        ivals.add((2, 3));
        assert_eq!(ivals.to_vec(), [(0, 1), (2, 3)]);
        ivals.add((1, 2));
        assert_eq!(ivals.to_vec(), [(0, 3)]);
    }

    /// Adding a smaller segment when a larger segment is already present with the same start.
    #[test]
    fn test_smaller_on_larger() {
        let mut ivals = CoalescedIntervals::<i64>::new();
        ivals.add((0, 3));
        assert_eq!(ivals.to_vec(), [(0, 3)]);
        ivals.add((0, 1));
        assert_eq!(ivals.to_vec(), [(0, 3)]);
    }

    /// Partial overlap between earlier and subsequent.
    #[test]
    fn test_partial_overlap() {
        let _ = env_logger::try_init();
        let mut ivals = CoalescedIntervals::<i64>::new();
        ivals.add((0, 3));
        assert_eq!(ivals.to_vec(), [(0, 3)]);
        ivals.add((2, 4));
        assert_eq!(ivals.to_vec(), [(0, 4)]);
    }
}
