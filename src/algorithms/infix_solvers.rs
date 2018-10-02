extern crate rand;
extern crate rayon as real_rayon;
use algorithms::infix_solvers::real_rayon::prelude::ParallelSlice;
use rayon::prelude::*;
use {Divisible, EdibleSlice, Policy};

pub struct InfixSlice<'a> {
    input: EdibleSlice<'a, Token>,
    output: PartialProducts,
}
#[derive(Debug)]
pub struct PartialProducts {
    products: Vec<u64>,
}

impl PartialProducts {
    fn new() -> Self {
        PartialProducts { products: vec![1] }
    }
    fn fuse(mut self, other: Self) -> Self {
        *self.products.last_mut().unwrap() *= other.products.first().unwrap();
        self.products.extend(&other.products[1..]);
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
            self.products = vec![
                self.products[0],
                sum,
                self.products[self.products.len() - 1],
            ];
        }
    }
}

impl<'a> Divisible for InfixSlice<'a> {
    fn len(&self) -> usize {
        self.input.len()
    }
    fn split(self) -> (Self, Self) {
        let (left_part, right_part) = self.input.split();
        (
            InfixSlice {
                input: left_part,
                output: self.output,
            },
            InfixSlice {
                input: right_part,
                output: PartialProducts::new(),
            },
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Mult,
    Add,
    Num(u64),
}

pub fn vec_gen() -> Vec<Token> {
    let size = 4_000_000;
    let expr = (1..size)
        .enumerate()
        .map(|(pos, num)| {
            if pos % 2 == 0 {
                Token::Num(num % 5)
            } else {
                let temp: u32 = rand::random();
                if temp % 10 == 0 {
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
            ::algorithms::infix_solvers::real_rayon::prelude::IntoParallelIterator::into_par_iter(
                slice,
            ).filter_map(|tok| match tok {
                Token::Mult | Token::Add => None,
                Token::Num(i) => Some(i),
            }).product::<u64>()
        }).sum::<u64>()
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
    let input = InfixSlice {
        input: EdibleSlice::new(inp),
        output: PartialProducts::new(),
    };
    input
        .work(
            |mut input, limit| {
                infix(&mut input.input, &mut input.output, limit);
                input
            },
            |slice| slice.output,
            |left, right| left.fuse(right),
            policy,
        ).evaluate()
}
