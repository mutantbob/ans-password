use crate::asymmetric_numeral_system::{ANSDecode, WeightedSymbols};

pub trait SymbolEmitter<S> {
    fn emit_symbol(&self, ans: &mut ANSDecode) -> S;
}

//

pub struct UniformSymbolSet<S> {
    symbols: Vec<S>,
}

impl<T: Clone> SymbolEmitter<T> for UniformSymbolSet<T> {
    fn emit_symbol(&self, ans: &mut ANSDecode) -> T {
        let idx = ans.decode_uniform(self.symbols.len());
        self.symbols.get(idx).unwrap().clone()
    }
}
