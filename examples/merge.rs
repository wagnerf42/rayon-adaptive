//! Merge parallel iterator.
//! let's compare a lot of versions to figure out what really works best.
use rayon_adaptive::prelude::*;
use rayon_adaptive::IndexedPower;
use std::iter::repeat_with;
use time::precise_time_ns;

const SIZE: usize = 500_000;

trait ParallelMerge {
    type ParallelMergeIterator;
    fn parallel_merge(self, other: Self) -> Self::ParallelMergeIterator;
}

impl<'a, T: 'a> ParallelMerge for &'a [T] {
    type ParallelMergeIterator = PMerge<'a, T>;
    fn parallel_merge(self, other: Self) -> Self::ParallelMergeIterator {
        PMerge(Merge {
            slices: [self, other],
            indexes: [0, 0],
        })
    }
}

struct PMerge<'a, T: 'a>(Merge<'a, T>);

// careful, this does not split equal elements evenly
impl<'a, T: 'a + Ord> Divisible for PMerge<'a, T> {
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        Some(
            self.0.slices[0].len() + self.0.slices[1].len() - self.0.indexes[0] - self.0.indexes[1],
        )
    }
    fn divide_at(self, _index: usize) -> (Self, Self) {
        let sizes = self
            .0
            .slices
            .iter()
            .zip(self.0.indexes.iter())
            .map(|(s, i)| s.len() - *i)
            .collect::<Vec<_>>();
        let divide = (sizes[0] < sizes[1]) as usize;
        let half_index = self.0.indexes[divide] + sizes[divide] / 2;
        let (left_half, right_half) = self.0.slices[divide].split_at(half_index);
        let uneven_index = self.0.indexes[1 - divide]
            + match self.0.slices[1 - divide][self.0.indexes[1 - divide]..]
                .binary_search(left_half.last().unwrap())
            {
                Ok(i) => i,
                Err(i) => i,
            };
        let (left_uneven, right_uneven) = self.0.slices[1 - divide].split_at(uneven_index);
        let left_slices = if divide == 0 {
            [left_half, left_uneven]
        } else {
            [left_uneven, left_half]
        };
        let right_slices = if divide == 0 {
            [right_half, right_uneven]
        } else {
            [right_uneven, right_half]
        };
        let left_indexes = if divide == 0 {
            [half_index, uneven_index]
        } else {
            [uneven_index, half_index]
        };
        (
            PMerge(Merge {
                slices: left_slices,
                indexes: left_indexes,
            }),
            PMerge(Merge {
                slices: right_slices,
                indexes: [0; 2],
            }),
        )
    }
}

struct Merge<'a, T: 'a> {
    slices: [&'a [T]; 2],
    indexes: [usize; 2],
}

impl<'a, T: 'a> Merge<'a, T> {
    fn advance_on(&mut self, side: usize) -> Option<&'a T> {
        let r = Some(&self.slices[side][self.indexes[side]]);
        self.indexes[side] += 1;
        r
    }
}

impl<'a, T: 'a + Ord> Iterator for Merge<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let slice1_is_empty = self.indexes[0] >= self.slices[0].len();
        let slice2_is_empty = self.indexes[1] >= self.slices[1].len();
        if !slice1_is_empty && !slice2_is_empty {
            if self.slices[0][self.indexes[0]] <= self.slices[1][self.indexes[1]] {
                self.advance_on(0)
            } else {
                self.advance_on(1)
            }
        } else if !slice1_is_empty {
            self.advance_on(0)
        } else if !slice2_is_empty {
            self.advance_on(1)
        } else {
            None
        }
    }
}

fn bench<R, I, S, F>(setup: S, f: F) -> u64
where
    S: Fn() -> I,
    F: Fn(I) -> R,
{
    repeat_with(|| {
        let start = precise_time_ns();
        let input = setup();
        f(input);
        let end = precise_time_ns();
        end - start
    })
    .take(1000)
    .sum::<u64>()
        / 1_000_000
}

fn main() {
    let t = bench(
        || {
            let (v1, v2): (Vec<u32>, Vec<u32>) =
                (0..((SIZE / 2) as u32)).map(|i| (2 * i, 2 * i + 1)).unzip();
            (v1, v2)
        },
        |(v1, v2)| {
            let m = Merge {
                slices: [v1.as_slice(), v2.as_slice()],
                indexes: [0, 0],
            };
            let r: Vec<u32> = m.copied().collect();
            assert_eq!(r.first(), Some(&0));
            assert_eq!(r.last(), Some(&((SIZE - 1) as u32)));
            (r, v1, v2)
        },
    );
    println!("it took with the merge iterator {}", t);
    let t = bench(
        || {
            let (v1, v2): (Vec<u32>, Vec<u32>) =
                (0..((SIZE / 2) as u32)).map(|i| (2 * i, 2 * i + 1)).unzip();
            (v1, v2)
        },
        |(v1, v2)| {
            let m = Merge {
                slices: [v1.as_slice(), v2.as_slice()],
                indexes: [0, 0],
            };
            let mut r = Vec::with_capacity(SIZE);
            r.extend(m.copied());
            assert_eq!(r.first(), Some(&0));
            assert_eq!(r.last(), Some(&((SIZE - 1) as u32)));
            (r, v1, v2)
        },
    );
    println!("it took with the merge iterator and capacity {}", t);
    let t = bench(
        || {
            let (v1, v2): (Vec<u32>, Vec<u32>) =
                (0..((SIZE / 2) as u32)).map(|i| (2 * i, 2 * i + 1)).unzip();
            (v1, v2)
        },
        |(v1, v2)| {
            let mut r = Vec::with_capacity(SIZE);
            unsafe { r.set_len(SIZE) };

            let mut left_index = 0;
            let mut right_index = 0;
            let mut output_index = 0;
            for _ in 0..SIZE {
                if v2.len() <= right_index {
                    //Go left all the way.
                    r[output_index..].copy_from_slice(&v1[left_index..]);
                    break;
                }
                if v1.len() <= left_index {
                    r[output_index..].copy_from_slice(&v2[right_index..]);
                    break;
                }
                let output_ref = unsafe { r.get_unchecked_mut(output_index) };
                output_index += 1;
                *output_ref = unsafe {
                    if v1.get_unchecked(left_index) <= v2.get_unchecked(right_index) {
                        let temp = v1.get_unchecked(left_index);
                        left_index += 1;
                        *temp
                    } else {
                        let temp = v2.get_unchecked(right_index);
                        right_index += 1;
                        *temp
                    }
                }
            }

            assert_eq!(r.first(), Some(&0));
            assert_eq!(r.last(), Some(&((SIZE - 1) as u32)));
            (r, v1, v2)
        },
    );
    println!("it took manually {}", t);
    let t = bench(
        || {
            let (v1, v2): (Vec<u32>, Vec<u32>) =
                (0..((SIZE / 2) as u32)).map(|i| (2 * i, 2 * i + 1)).unzip();
            (v1, v2)
        },
        |(v1, v2)| {
            let mut r = Vec::with_capacity(SIZE);
            let mut i1 = v1.iter().rev().copied().peekable();
            let mut i2 = v2.iter().rev().copied().peekable();
            for _ in 0..SIZE {
                r.extend(if i1.peek() > i2.peek() {
                    i1.next()
                } else {
                    i2.next()
                });
            }

            assert_eq!(r.last(), Some(&0));
            assert_eq!(r.first(), Some(&((SIZE - 1) as u32)));
            (r, v1, v2)
        },
    );
    println!("it took with peek {}", t);

    // let merged:Vec<u32> = v1.parallel_merge(&v2).collect();
}
