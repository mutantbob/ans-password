use crate::required_symbols::ClassWeight;
use crate::symbol_generator::SymbolEmitter;
use crate::{DIGITS, LOWERS, MISC, UPPERS};
use std::rc::Rc;

pub struct ANSDecode<'a> {
    byte_source: Box<dyn Iterator<Item = u8> + 'a>,
    state: u64,
}

impl<'s> ANSDecode<'s> {
    pub fn new<I: Iterator<Item = u8> + 's>(mut byte_source: I) -> Self {
        let state = (0..8).fold(0, |a, i| {
            a | ((byte_source.next().unwrap() as u64) << (8 * i))
        });

        Self {
            byte_source: Box::new(byte_source),
            state,
        }
    }

    pub fn decode_uniform(&mut self, modulus: usize) -> usize {
        let rem = self.state % (modulus as u64);
        let x = self.state / (modulus as u64);
        self.set_state_maybe_load(x);
        rem as usize
    }

    pub fn decode_uniform_from<'a, S>(&mut self, symbols: &'a [S]) -> &'a S {
        &symbols[self.decode_uniform(symbols.len())]
    }

    pub fn decode_from_weights<'a, S>(&mut self, weights: &'a WeightedSymbols<S>) -> &'a S {
        let (symbol, new_state) = weights.do_ans(self.state);
        self.set_state_maybe_load(new_state);
        symbol
    }

    /// # returns
    /// false if it chooses the `a` weight; true if it chooses the `b` weight
    pub fn decode_binary(&mut self, a: f32, b: f32, max_quantization: u64) -> bool {
        let sum = a + b;
        let (a1, b, sum) = if a + b > max_quantization as f32 {
            let a = (a * max_quantization as f32 / sum) as u64;
            let b = max_quantization - a;
            (a, b, max_quantization)
        } else {
            let a = a as u64;
            let b = b as u64;
            (a, b, a + b)
        };
        let rem = self.state % sum;
        let x = self.state / sum;
        if rem < a1 {
            self.set_state_maybe_load(x * a1 + rem);
            false
        } else {
            self.set_state_maybe_load(x * b + (rem - a1));
            true
        }
    }

    fn set_state_maybe_load(&mut self, mut new_state: u64) {
        // println!("debug state {:x} -> {:x}", self.state, new_state);
        if new_state < 1 << 56 {
            // println!("fetch byte");
            if let Some(val) = self.byte_source.next() {
                new_state |= (val as u64) << 56;
            }
        }
        self.state = new_state;
    }
}

//

pub struct WeightedSymbols<S> {
    weight_sum: u64,
    weights: Vec<u32>,
    offsets: Vec<u64>,
    symbols: Vec<S>,
}

impl<S> WeightedSymbols<S> {
    pub fn bob() -> WeightedSymbols<SimpleClass> {
        WeightedSymbols::new(&[
            ClassWeight::new(SimpleClass::Upper, 5),
            ClassWeight::new(SimpleClass::Lower, 5),
            ClassWeight::new(SimpleClass::Digit, 1),
            ClassWeight::new(SimpleClass::Misc, 1),
        ])
    }

    pub fn bob2() -> Weighted2Stage<char> {
        type F1 = Rc<dyn Fn(&mut ANSDecode) -> char>;
        let fn_u: F1 = Rc::new(|ans: &mut ANSDecode| *ans.decode_uniform_from(&UPPERS));
        let fn_l: F1 = Rc::new(|ans: &mut ANSDecode| *ans.decode_uniform_from(&LOWERS));
        let fn_d: F1 = Rc::new(|ans: &mut ANSDecode| *ans.decode_uniform_from(&DIGITS));
        let fn_m: F1 = Rc::new(|ans: &mut ANSDecode| *ans.decode_uniform_from(&MISC));
        let syms = WeightedSymbols::new(&[
            ClassWeight::new(fn_u, 5),
            ClassWeight::new(fn_l, 5),
            ClassWeight::new(fn_d, 1),
            ClassWeight::new(fn_m, 1),
        ]);

        Weighted2Stage { layer1: syms }
    }
}

impl<S: Clone> WeightedSymbols<S> {
    pub fn new<'a>(dictionary: impl IntoIterator<Item = &'a ClassWeight<S>>) -> Self
    where
        S: 'a,
    {
        let mut weight_sum = 0u64;
        let mut offsets = vec![];
        let mut weights = vec![];
        let mut symbols = vec![];
        for cw in dictionary {
            offsets.push(weight_sum);
            weights.push(cw.weight);
            weight_sum += cw.weight as u64;
            symbols.push(cw.class.clone());
        }
        Self {
            weight_sum,
            weights,
            offsets,
            symbols,
        }
    }
}

impl<S> WeightedSymbols<S> {
    pub fn do_ans(&self, state: u64) -> (&S, u64) {
        let rem = state % self.weight_sum;
        let x = state / self.weight_sum;

        let i = self.find_bin(rem);
        let phase = rem - self.offsets[i];
        let symbol = &self.symbols[i];
        let new_state = x * (self.weights[i] as u64) + phase;
        (symbol, new_state)
    }

    pub fn find_bin(&self, remainder: u64) -> usize {
        for (idx, offset) in self.offsets.iter().enumerate() {
            if *offset > remainder {
                return idx - 1;
            }
        }
        self.offsets.len() - 1
    }
}

impl<'s: 'r, 'r, S> SymbolEmitter<'s, &'r S> for WeightedSymbols<S> {
    fn emit_symbol(&'s mut self, ans: &mut ANSDecode) -> Option<&'r S> {
        Some(ans.decode_from_weights(self))
    }
}

//

pub struct Weighted2Stage<T> {
    #[allow(clippy::type_complexity)]
    layer1: WeightedSymbols<Rc<dyn Fn(&mut ANSDecode) -> T>>,
}

impl<T> Weighted2Stage<T> {
    #[allow(clippy::type_complexity)]
    pub fn new(classes: WeightedSymbols<Rc<dyn Fn(&mut ANSDecode) -> T>>) -> Self {
        Self { layer1: classes }
    }
}

impl<T> SymbolEmitter<'_, T> for Weighted2Stage<T> {
    fn emit_symbol(&mut self, ans: &mut ANSDecode) -> Option<T> {
        let stage2 = ans.decode_from_weights(&self.layer1);
        Some(stage2(ans))
    }
}

//

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SimpleClass {
    Upper,
    Lower,
    Digit,
    Misc,
}
