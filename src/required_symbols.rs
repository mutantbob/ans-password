use crate::asymmetric_numeral_system::ANSDecode;
use crate::symbol_generator::SymbolEmitter;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

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

    /// what are the probabilities for the required and optional symbols given that
    /// there must be at least `required_count` required symbol in the next `remaining_symbols` symbols
    /// # returns
    /// (required_weight, optional_weight)
    pub fn weights_general(&self, required_count: u32, remaining_count: u32) -> (f32, f32) {
        if required_count == 0 {
            self.unrestricted_weight(remaining_count)
        } else if remaining_count <= required_count {
            (self.required_prev.powi(remaining_count as i32), 0.0)
        } else {
            let fewer = remaining_count - 1;
            let (r_w, o_w) = self.weights_general(required_count - 1, fewer);
            let w_a = r_w + o_w;
            let (r_w, o_w) = self.weights_general(required_count, fewer);
            let w_b = r_w + o_w;
            (self.required_prev * w_a, self.optional_prev * w_b)
        }
    }

    /// The results are intentionally scaled so they can be used inside restricted_weight
    pub fn unrestricted_weight(&self, remaining_symbols: u32) -> (f32, f32) {
        let sum = self.required_prev + self.optional_prev;
        let pow = Self::pow1(sum, remaining_symbols.max(1) as i32 - 1);
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

pub struct SymbolsWithRequirement<S> {
    adjustmotron: Adjustmotron,
    required_count: u32,
    remaining_symbols: u32,
    #[allow(clippy::type_complexity)]
    fn_a: Rc<RefCell<dyn FnMut(&mut ANSDecode) -> S>>,
    #[allow(clippy::type_complexity)]
    fn_b: Rc<RefCell<dyn FnMut(&mut ANSDecode) -> S>>,
    phantom_data: PhantomData<S>,
}

impl<S> SymbolsWithRequirement<S> {
    pub fn new<FA, FB>(
        required_prev: f32,
        optional_prev: f32,
        required_count: u32,
        remaining_symbols: u32,
        fn_a: FA,
        fn_b: FB,
    ) -> Self
    where
        FA: FnMut(&mut ANSDecode) -> S + 'static,
        FB: FnMut(&mut ANSDecode) -> S + 'static,
    {
        Self {
            adjustmotron: Adjustmotron::new(required_prev, optional_prev),
            required_count,
            remaining_symbols,
            fn_a: Rc::new(RefCell::new(fn_a)),
            fn_b: Rc::new(RefCell::new(fn_b)),
            phantom_data: Default::default(),
        }
    }
}

impl<S> SymbolEmitter<'_, S> for SymbolsWithRequirement<S> {
    fn emit_symbol(&mut self, ans: &mut ANSDecode) -> S {
        let (a, b) = self
            .adjustmotron
            .weights_general(self.required_count, self.remaining_symbols);
        let use_b = ans.decode_binary(a, b, 1 << 32);

        self.remaining_symbols = self.remaining_symbols.max(1) - 1;

        if use_b {
            (self.fn_b.borrow_mut())(ans)
        } else {
            if self.required_count > 0 {
                self.required_count -= 1;
            }
            (self.fn_a.borrow_mut())(ans)
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
            let (a, b) = atron.unrestricted_weight(1);
            assert_eq!(3.0, a);
            assert_eq!(4.0, b);
        }

    }

    #[test]
    pub fn test2() {
        let atron = Adjustmotron::new(3.0, 4.0);

        {
            let (a, b) = atron.weights_general(1, 1);
            assert_eq!(3.0, a);
            assert_eq!(0.0, b);
        }
    }

    #[test]
    pub fn test3() {
        {
            let (a, b) = Adjustmotron::new(1.0, 1.0).weights_general(1, 2);
            assert_eq!(2.0, a);
            assert_eq!(1.0, b);
        }

        {
            let (a, b) = Adjustmotron::new(2.0, 2.0).weights_general(1, 2);
            assert_eq!(2.0 * 4.0, a);
            assert_eq!(2.0 * 2.0, b);
        }

        {
            let (a, b) = Adjustmotron::new(3.0, 4.0).weights_general(1, 2);
            assert_eq!(3.0 * (4.0 + 3.0), a);
            assert_eq!(4.0 * 3.0, b);
        }
    }

    #[test]
    pub fn test4() {
        {
            let (a, b) = Adjustmotron::new(1.0, 1.0).weights_general(1, 8);
            assert_eq!(128.0, a);
            assert_eq!(127.0, b);
        }
    }

    #[test]
    pub fn test5() {
        let a11 = Adjustmotron::new(1.0, 1.0);
        {
            let (a, b) = a11.weights_general(2, 2);
            assert_eq!(1.0, a);
            assert_eq!(0.0, b);
        }
        {
            let (a, b) = a11.weights_general(2, 3);
            assert_eq!(3.0, a);
            assert_eq!(1.0, b);
        }
        {
            let (a, b) = a11.weights_general(2, 4);
            assert_eq!(7.0, a);
            assert_eq!(4.0, b);
        }
        {
            let (a, b) = a11.weights_general(2, 5);
            assert_eq!(15.0, a);
            assert_eq!(11.0, b);
        }
        {
            let (a, b) = a11.weights_general(2, 6);
            assert_eq!(31.0, a);
            assert_eq!(26.0, b);
        }
        {
            let (a, b) = a11.weights_general(2, 7);
            assert_eq!(63.0, a);
            assert_eq!((64 - 7) as f32, b);
        }

        {
            let (a, b) = a11.weights_general(2, 8);
            assert_eq!(127.0, a);
            assert_eq!(120.0, b);
        }
    }
}
