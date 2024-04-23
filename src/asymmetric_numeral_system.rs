use crate::required_symbols::ClassWeight;

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

//

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SimpleClass {
    Upper,
    Lower,
    Digit,
    Misc,
}
