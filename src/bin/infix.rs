extern crate rand;
extern crate rayon_adaptive;
extern crate rayon_logs;
extern crate time;
use rayon_adaptive::*;
use rayon_logs::ThreadPoolBuilder;
use time::*;
use {Divisible, EdibleSlice, Policy};

struct InfixSlice<'a> {
    input: EdibleSlice<'a, Token>,
    output: PartialProducts,
}

struct PartialProducts {
    current_product: u64,
    first_product: u64,
    sum_of_intermediate_products: u64,
    addition_met: bool,
}

impl PartialProducts {
    fn new() -> Self {
        PartialProducts {
            current_product: 1,
            first_product: 0,
            sum_of_intermediate_products: 0,
            addition_met: false,
        }
    }
    fn evaluate(self) -> u64 {
        self.first_product + self.sum_of_intermediate_products + self.current_product
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
    fn fuse(self, right: Self) -> Self {
        if right.addition_met {
            PartialProducts {
                current_product: right.current_product,
                first_product: self.first_product,
                sum_of_intermediate_products: self.sum_of_intermediate_products
                    + self.current_product * right.first_product
                    + right.sum_of_intermediate_products,
                addition_met: true,
            }
        } else {
            PartialProducts {
                current_product: self.current_product * right.current_product,
                first_product: self.first_product,
                sum_of_intermediate_products: self.sum_of_intermediate_products,
                addition_met: self.addition_met,
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Mult,
    Add,
    Num(u64),
}

pub fn vec_gen() -> Vec<Token> {
    let size = 1_000_000;
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

pub fn solver_seq(inp: &[Token]) -> u64 {
    let ans = inp.iter().fold((0, 1), |tup, elem| match elem {
        Token::Add => (tup.0 + tup.1, 1),
        Token::Mult => tup,
        Token::Num(i) => (tup.0, tup.1 * *i),
    });
    ans.0 + ans.1
}

//this is the work function
fn infix(input_slice: &mut EdibleSlice<Token>, output: &mut PartialProducts, limit: usize) {
    input_slice.iter().take(limit).for_each(|tok| match tok {
        Token::Num(i) => output.current_product *= i,
        Token::Add => {
            if output.addition_met {
                output.sum_of_intermediate_products += output.current_product;
            } else {
                output.addition_met = true;
                output.first_product = output.current_product;
            }
            output.current_product = 1;
        }
        Token::Mult => {}
    });
}

fn wrapper(inp: &Vec<Token>, policy: Policy) -> u64 {
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
        .num_threads(3)
        .build()
        .expect("Pool creation failed");

    let answer = solver_seq(&testin);

    /*pool.compare(
        "sequential",
        "adaptive",
        || {
            let count = solver_seq(&testin);
            assert_eq!(count, answer);
        },
        || {
            let count = wrapper(&testin, Policy::Adaptive(1000));
            assert_eq!(count, answer);
        },
        "seq_adapt.html",
    ).expect("logging failed");*/

    let par_time_start = precise_time_ns();
    let (par_out, log) = pool.install(|| wrapper(&testin, Policy::Adaptive(100_000)));
    let par_time_end = precise_time_ns();
    let seq_time_start = precise_time_ns();
    let seq_out = solver_seq(&testin);
    let seq_time_end = precise_time_ns();
    let seq_time_start = seq_time_end - seq_time_start;
    let par_time_start = par_time_end - par_time_start;
    assert_eq!(par_out, seq_out);
    println!(
        "seq time {}\npar time {}",
        (seq_time_start as f64) / 1_000_000.0,
        (par_time_start as f64) / 1_000_000.0
    );
    log.save_svg("infix.svg").expect("saving failed");
}
