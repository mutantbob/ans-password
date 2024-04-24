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
        ans.decode_uniform_from(&self.symbols).clone()
    }
}

impl<'s, T: Clone, U: AsRef<[T]>> SymbolEmitter<'s, T> for U {
    fn emit_symbol(&'s mut self, ans: &mut ANSDecode) -> T {
        ans.decode_uniform_from(self.as_ref()).clone()
    }
}
