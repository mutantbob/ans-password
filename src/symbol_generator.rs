use crate::asymmetric_numeral_system::ANSDecode;

pub trait SymbolEmitter<'s, S> {
    fn emit_symbol(&'s mut self, ans: &mut ANSDecode) -> S;
}

//

pub struct UniformSymbolSet<S> {
    symbols: Vec<S>,
}

impl<'s, T: Clone> SymbolEmitter<'s, T> for UniformSymbolSet<T> {
    fn emit_symbol(&'s mut self, ans: &mut ANSDecode) -> T {
        let idx = ans.decode_uniform(self.symbols.len());
        self.symbols.get(idx).unwrap().clone()
    }
}
