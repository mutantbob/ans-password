use crate::asymmetric_numeral_system::ANSDecode;
use crate::symbol_generator::SymbolEmitter;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct ClassWeight<T: Clone> {
    pub class: T,
    pub weight: u32,
    // minimum: u32,
}

impl<T: Clone> ClassWeight<T> {
    pub fn new(class: T, weight: u32) -> Self {
        Self {
            class,
            weight,
            // minimum,
        }
    }
}

//

/// Tool for calculating the probability of a symbol when symbols can be divided into a *required* and *optional* category.
/// When a symbol is required to appear in a string, that distorts the probability of each symbol in a slot.
pub struct Adjustmotron {
    required_prev: f32,
    optional_prev: f32,
}

impl Adjustmotron {
    /// `required_prev` - the raw weighting of the required symbol
    /// `optional_prev` - the raw weight of the optional symbols
    pub fn new(required_prev: f32, optional_prev: f32) -> Self {
        Self {
            required_prev,
            optional_prev,
        }
    }

    /// what are the probabilities for the required and optional symbols given that there must be at least one required symbol in the next `remaining_symbols` symbols
    /// # returns
    /// (required_weight, optional_weight)
    pub fn restricted_weight(&self, remaining_symbols: u32) -> (f32, f32) {
        if remaining_symbols <= 1 {
            (self.required_prev, 0.0)
        } else {
            let fewer = remaining_symbols - 1;
            let (r_w, o_w) = self.weight(fewer);
            let w_a = r_w + o_w;
            let (r_w, o_w) = self.restricted_weight(fewer);
            let w_b = r_w + o_w;
            (self.required_prev * w_a, self.optional_prev * w_b)
        }
    }

    /// The results are intentionally scaled so they can be used inside restricted_weight
    pub fn weight(&self, remaining_symbols: u32) -> (f32, f32) {
        let sum = self.required_prev + self.optional_prev;
        let pow = Self::pow1(sum, remaining_symbols as i32 - 1);
        (self.required_prev * pow, self.optional_prev * pow)
    }

    /// in python, this would be x**n.  LaTeX would use x^n
    fn pow1(x: f32, n: i32) -> f32 {
        x.powi(n)
        // (0..n).fold(1.0, |a, _b| a * x)
    }
}

//
//

pub struct SymbolsWithRequirement<S, FA, FB>
where
    FA: Fn() -> S,
    FB: Fn() -> S,
{
    adjustmotron: Adjustmotron,
    satisfied: bool,
    remaining_symbols: u32,
    fn_a: FA,
    fn_b: FB,
    phantom_data: PhantomData<S>,
}

impl<S, FA: Fn() -> S, FB: Fn() -> S> SymbolsWithRequirement<S, FA, FB> {
    pub fn new(
        required_prev: f32,
        optional_prev: f32,
        remaining_symbols: u32,
        fn_a: FA,
        fn_b: FB,
    ) -> Self {
        Self {
            adjustmotron: Adjustmotron::new(required_prev, optional_prev),
            satisfied: false,
            remaining_symbols,
            fn_a,
            fn_b,
            phantom_data: Default::default(),
        }
    }
}

impl<S, FA: Fn() -> S, FB: Fn() -> S> SymbolEmitter<'_, S> for SymbolsWithRequirement<S, FA, FB> {
    fn emit_symbol(&mut self, ans: &mut ANSDecode) -> S {
        let (a, b) = if self.satisfied {
            self.adjustmotron.weight(1)
        } else {
            self.adjustmotron.restricted_weight(self.remaining_symbols)
        };
        let x = ans.decode_binary(a, b, 1 << 32);

        self.remaining_symbols -= 1;

        if x {
            self.satisfied = true;
            (self.fn_a)()
        } else {
            (self.fn_b)()
        }
    }
}

#[cfg(test)]
mod test {
    use super::Adjustmotron;

    #[test]
    pub fn test1() {
        let atron = Adjustmotron::new(3.0, 4.0);
        {
            let (a, b) = atron.weight(1);
            assert_eq!(3.0, a);
            assert_eq!(4.0, b);
        }

        {
            let (a, b) = atron.restricted_weight(1);
            assert_eq!(3.0, a);
            assert_eq!(0.0, b);
        }
    }

    #[test]
    pub fn test2() {
        {
            let (a, b) = Adjustmotron::new(1.0, 1.0).restricted_weight(2);
            assert_eq!(2.0, a);
            assert_eq!(1.0, b);
        }

        {
            let (a, b) = Adjustmotron::new(2.0, 2.0).restricted_weight(2);
            assert_eq!(2.0 * 4.0, a);
            assert_eq!(2.0 * 2.0, b);
        }

        {
            let (a, b) = Adjustmotron::new(3.0, 4.0).restricted_weight(2);
            assert_eq!(3.0 * (4.0 + 3.0), a);
            assert_eq!(4.0 * 3.0, b);
        }
    }

    #[test]
    pub fn test3() {
        {
            let (a, b) = Adjustmotron::new(1.0, 1.0).restricted_weight(8);
            assert_eq!(128.0, a);
            assert_eq!(127.0, b);
        }
    }
}
