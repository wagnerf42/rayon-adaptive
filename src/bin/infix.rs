extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;
extern crate rayon_logs;
extern crate time;
use rayon::prelude::ParallelSlice;
use rayon_adaptive::*;
use rayon_logs::ThreadPoolBuilder;
use rayon_logs::prelude::*;
use time::*;
use {Divisible, EdibleSlice, Policy};

struct InfixSlice<'a> {
    input: EdibleSlice<'a, Token>,
    output: PartialProducts,
}
#[derive(Debug)]
struct PartialProducts {
    products: Vec<u64>,
    len: usize,
}

impl PartialProducts {
    fn new() -> Self {
        PartialProducts {
            products: vec![1],
            len: 1,
        }
    }
    fn evaluate(self) -> u64 {
        self.products.iter().sum::<u64>()
    }
    fn update_product(&mut self, num: u64) {
        let len: usize = self.len;
        self.products[len - 1] *= num;
    }
    fn append_product(&mut self) {
        self.products.push(1);
        self.len += 1;
    }
    fn reduce_products(&mut self) {
        if self.len <= 3 {
            ()
        } else {
            let sum: u64 = self.products[1..self.len - 1].iter().sum::<u64>();
            self.products = vec![self.products[0], sum, self.products[self.len - 1]];
            self.len = 3;
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

impl Mergeable for PartialProducts {
    fn fuse(mut self, right: Self) -> Self {
        let leftlen = self.len;
        let rightlen = right.len;
        let join_elem: u64 = self.products[leftlen - 1] * right.products[0];
        self.products[leftlen - 1] = join_elem;
        let result = self
            .products
            .into_iter()
            .chain(right.products.into_iter().skip(1))
            .collect::<Vec<u64>>();
        let mut res = PartialProducts {
            products: result,
            len: leftlen + rightlen - 1,
        };
        res.reduce_products();
        res
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Mult,
    Add,
    Num(u64),
}

fn vec_gen() -> Vec<Token> {
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
        })
        .collect::<Vec<Token>>();
    expr
}

fn solver_seq(inp: &[Token]) -> u64 {
    let ans = inp.iter().fold((0, 1), |tup, elem| match elem {
        Token::Add => (tup.0 + tup.1, 1),
        Token::Mult => tup,
        Token::Num(i) => (tup.0, tup.1 * *i),
    });
    ans.0 + ans.1
}

fn solver_par_split(inp: &[Token]) -> u64 {
    inp.as_parallel_slice()
        .par_split(|tok| *tok == Token::Add)
        .map(|slice| {
            slice
                .iter()
                .filter_map(|tok| match tok {
                    Token::Mult | Token::Add => None,
                    Token::Num(i) => Some(i),
                })
                .product::<u64>()
        })
        .sum::<u64>()
}

fn solver_par_fold(inp: &[Token]) -> u64 {
    inp.par_iter()
        .fold(
            || PartialProducts::new(),
            |mut products, tok| match *tok {
                Token::Add => {
                    products.append_product();
                    products
                }
                Token::Num(i) => {
                    products.update_product(i);
                    products
                }
                Token::Mult => products,
            },
        )
        .reduce(|| PartialProducts::new(), |left, right| left.fuse(right))
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

fn solver_adaptive(inp: &Vec<Token>, policy: Policy) -> u64 {
    let input = InfixSlice {
        input: EdibleSlice::new(inp),
        output: PartialProducts::new(),
    };
    input
        .work(
            |input, limit| infix(&mut input.input, &mut input.output, limit),
            |slice| slice.output,
            policy,
        )
        .evaluate()
}

fn main() {
    let testin = vec_gen();
    let pool = ThreadPoolBuilder::new()
        .num_threads(8)
        .build()
        .expect("Pool creation failed");

    let answer = solver_seq(&testin);

    pool.compare(
        "sequential",
        "adaptive",
        || {
            let count = solver_seq(&testin);
            assert_eq!(count, answer);
        },
        || {
            let count = solver_adaptive(&testin, Policy::Adaptive(1000));
            assert_eq!(count, answer);
        },
        "seq_adapt.html",
    ).expect("logging failed");

    pool.compare(
        "adaptive",
        "rayon split",
        || {
            let count = solver_adaptive(&testin, Policy::Adaptive(1000));
            assert_eq!(count, answer);
        },
        || {
            let count = solver_par_split(&testin);
            assert_eq!(count, answer);
        },
        "adapt_split.html",
    ).expect("logging failed");

    pool.compare(
        "adaptive",
        "rayon fold",
        || {
            let count = solver_adaptive(&testin, Policy::Adaptive(1000));
            assert_eq!(count, answer);
        },
        || {
            let count = solver_par_fold(&testin);
            assert_eq!(count, answer);
        },
        "adapt_fold.html",
    ).expect("logging failed");

    //  let seq_time_start = precise_time_ns();
    //  let seq_out = solver_seq(&testin);
    //  let seq_time_end = precise_time_ns();

    //  let par_time_start = precise_time_ns();
    //  let (paradaptive_out, log) = pool.install(|| solver_adaptive(&testin, Policy::Adaptive(9000)));
    //  let par_time_end = precise_time_ns();

    //  let parsplit_time_start = precise_time_ns();
    //  let parsplit_out = solver_par_split(&testin);
    //  let parsplit_time_end = precise_time_ns();

    //  let parfold_time_start = precise_time_ns();
    //  let parfold_out = solver_par_fold(&testin);
    //  let parfold_time_end = precise_time_ns();

    //  let seq_time_start = seq_time_end - seq_time_start;
    //  let par_time_start = par_time_end - par_time_start;
    //  let parsplit_time_start = parsplit_time_end - parsplit_time_start;
    //  let parfold_time_start = parfold_time_end - parfold_time_start;
    //  assert_eq!(paradaptive_out, seq_out);
    //  assert_eq!(parfold_out, seq_out);
    //  assert_eq!(parsplit_out, seq_out);
    //  println!(
    //      "seq time {}\nadaptive par time {}\nparsplit time {}\nparfold time {}",
    //      (seq_time_start as f64) / 1_000_000.0,
    //      (par_time_start as f64) / 1_000_000.0,
    //      (parsplit_time_start as f64) / 1_000_000.0,
    //      (parfold_time_start as f64) / 1_000_000.0
    //  );
    //  log.save_svg("infix.svg").expect("saving failed");
}
