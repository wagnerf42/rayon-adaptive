extern crate rand;
use crate::prelude::*;
use crate::rayon::prelude::*;
use crate::Policy;
use smallvec::SmallVec;
#[derive(Debug, Clone)]
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
    fn fuse(mut self, other: &Self) -> Self {
        *self.products.last_mut().unwrap() *= other.products.first().unwrap();
        self.products.extend(other.products[1..].iter().cloned());
        self.reduce_products();
        self
    }
    fn evaluate(self) -> u64 {
        self.products.iter().sum::<u64>()
    }
    fn update_product(&mut self, num: u64) {
        *self.products.last_mut().unwrap() *= num;
        //let len = self.products.len();
        //self.products[len - 1] *= num;
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
    (1..size)
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
        })
        .collect()
}

pub fn solver_seq(inp: &[Token]) -> u64 {
    let t = inp.iter().fold((0, 1), |tup, elem| match elem {
        Token::Num(i) => (tup.0, tup.1 * *i),
        Token::Mult => tup,
        Token::Add => (tup.0 + tup.1, 1),
    });
    t.0 + t.1
}

//Not logged by rayon_logs.
pub fn solver_par_split(inp: &[Token]) -> u64 {
    #[cfg(feature = "logs")]
    use crate::real_rayon::prelude::ParallelSlice;
    inp.as_parallel_slice()
        .par_split(|tok| *tok == Token::Add)
        .map(|slice| {
            // It's tricky because rayon-logs does not support par_split right now
            #[cfg(not(feature = "logs"))]
            let iterator = slice.into_par_iter();
            #[cfg(feature = "logs")]
            let iterator = crate::real_rayon::prelude::IntoParallelIterator::into_par_iter(slice);
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
        .fold(PartialProducts::new, |mut products, tok| match *tok {
            Token::Num(i) => {
                products.update_product(i);
                products
            }
            Token::Add => {
                products.append_product();
                products
            }
            Token::Mult => products,
        })
        .reduce(PartialProducts::new, |left, right| left.fuse(&right))
        .evaluate()
}

pub fn solver_adaptive(inp: &[Token], policy: Policy) -> u64 {
    inp.into_adapt_iter()
        .with_policy(policy)
        .fold(PartialProducts::new, |mut p, token| {
            match token {
                Token::Num(i) => p.update_product(*i),
                Token::Add => p.append_product(),
                Token::Mult => {}
            }
            p
        })
        .reduce(|left, right| left.fuse(&right))
        .evaluate()
}

pub fn solver_fully_adaptive(inp: &[Token]) -> u64 {
    let (s, p) = inp
        .into_adapt_iter()
        .fold(PartialProducts::new, |mut p, token| {
            match token {
                Token::Num(i) => p.update_product(*i),
                Token::Add => p.append_product(),
                Token::Mult => {}
            }
            p
        })
        .helping_fold(
            (0, 1),
            |(s, p), token| match token {
                Token::Num(i) => (s, p * i),
                Token::Add => (s + p, 1),
                Token::Mult => (s, p),
            },
            |(s, p), pprod| match pprod.products.len() {
                1 => (s, p * pprod.products[0]),
                2 => (s + p * pprod.products[0], pprod.products[1]),
                3 => (
                    s + p * pprod.products[0] + pprod.products[1],
                    pprod.products[2],
                ),
                _ => panic!("no way"),
            },
        );
    s + p
}
