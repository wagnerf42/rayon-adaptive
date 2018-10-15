extern crate rand;

#[cfg(not(feature = "logs"))]
extern crate rayon;
#[cfg(not(feature = "logs"))]
use algorithms::infix_solvers::rayon::prelude::ParallelSlice;
#[cfg(feature = "logs")]
extern crate rayon as real_rayon;
#[cfg(feature = "logs")]
use algorithms::infix_solvers::real_rayon::prelude::ParallelSlice;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

use rayon::prelude::*;
use smallvec::SmallVec;

#[cfg(feature = "logs")]
use rayon::sequential_task;
use {Divisible, EdibleSlice, KeepLeft, Policy};

#[derive(Debug)]
pub struct PartialProducts {
    products: SmallVec<[u64; 3]>,
}

impl Default for PartialProducts {
    fn default() -> Self {
        PartialProducts::new()
    }
}

impl PartialProducts {
    fn new() -> Self {
        PartialProducts {
            products: smallvec![1],
        }
    }
    fn fuse(mut self, other: Self) -> Self {
        *self.products.last_mut().unwrap() *= other.products.first().unwrap();
        self.products.extend(other.products[1..].iter().cloned());
        self.reduce_products();
        self
    }
    fn evaluate(self) -> u64 {
        self.products.iter().sum::<u64>()
    }
    fn update_product(&mut self, num: u64) {
        let len = self.products.len();
        self.products[len - 1] *= num;
    }
    fn append_product(&mut self) {
        if self.products.len() == 3 {
            self.products[1] += self.products[2];
            self.products[2] = 1;
        } else {
            self.products.push(1);
        }
    }
    fn reduce_products(&mut self) {
        if self.products.len() > 3 {
            let sum: u64 = self.products[1..self.products.len() - 1]
                .iter()
                .sum::<u64>();
            self.products = smallvec![
                self.products[0],
                sum,
                self.products[self.products.len() - 1],
            ];
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Mult,
    Add,
    Num(u64),
}

pub fn vec_gen(size: u64) -> Vec<Token> {
    let expr = (1..size)
        .enumerate()
        .map(|(pos, num)| {
            if pos % 2 == 0 {
                Token::Num(num % 5)
            } else {
                let temp: u32 = rand::random();
                if temp % 100_000 == 0 {
                    Token::Add
                } else {
                    Token::Mult
                }
            }
        }).collect::<Vec<Token>>();
    expr
}

pub fn solver_seq(inp: &[Token]) -> u64 {
    let ans = inp.iter().fold((0, 1), |tup, elem| match elem {
        Token::Add => (tup.0 + tup.1, 1),
        Token::Mult => tup,
        Token::Num(i) => (tup.0, tup.1 * *i),
    });
    ans.0 + ans.1
}
//Not logged by rayon_logs.
pub fn solver_par_split(inp: &[Token]) -> u64 {
    inp.as_parallel_slice()
        .par_split(|tok| *tok == Token::Add)
        .map(|slice| {
            // It's tricky because rayon-logs does not support par_split right now
            #[cfg(not(feature = "logs"))]
            let iterator = slice.into_par_iter();
            #[cfg(feature = "logs")]
            let iterator = ::algorithms::infix_solvers::real_rayon::prelude::IntoParallelIterator::into_par_iter(slice);
            iterator
                .filter_map(|tok| match tok {
                    Token::Mult | Token::Add => None,
                    Token::Num(i) => Some(i),
                })
                .product::<u64>()
        })
        .sum::<u64>()
}
//Logged
pub fn solver_par_fold(inp: &[Token]) -> u64 {
    inp.into_par_iter()
        .fold(
            || PartialProducts::new(),
            |mut products, tok| match *tok {
                Token::Num(i) => {
                    products.update_product(i);
                    products
                }
                Token::Add => {
                    products.append_product();
                    products
                }
                Token::Mult => products,
            },
        ).reduce(|| PartialProducts::new(), |left, right| left.fuse(right))
        .evaluate()
}

//this is the work function
fn infix(input_slice: &mut EdibleSlice<Token>, output: &mut PartialProducts, limit: usize) {
    input_slice.iter().take(limit).for_each(|tok| match tok {
        Token::Num(i) => output.update_product(*i),
        Token::Add => output.append_product(),
        Token::Mult => {}
    });
}

pub fn solver_adaptive(inp: &Vec<Token>, policy: Policy) -> u64 {
    let input = (EdibleSlice::new(inp), KeepLeft(PartialProducts::new()));
    input
        .work(|mut input, limit| {
            #[cfg(feature = "logs")]
            sequential_task(0, limit, || infix(&mut input.0, &mut input.1, limit));
            #[cfg(not(feature = "logs"))]
            infix(&mut input.0, &mut input.1, limit);
            input
        }).map(|slice| (slice.1).0)
        .reduce(|left, right| left.fuse(right), policy)
        .evaluate()
}
