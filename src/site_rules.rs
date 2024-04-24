use crate::asymmetric_numeral_system::{ANSDecode, Weighted2Stage, WeightedSymbols};
use crate::required_symbols::{ClassWeight, SymbolsWithRequirement};
use crate::symbol_generator::SymbolEmitter;
use crate::{DIGITS, LOWERS, MISC, UPPERS};
use std::rc::Rc;

pub type F1 = Rc<dyn Fn(&mut ANSDecode) -> char>;

pub fn ericsson() -> SymbolsWithRequirement<char> {
    let fn_u: F1 = Rc::new(|ans: &mut ANSDecode| *ans.decode_uniform_from(&UPPERS));
    let fn_l: F1 = Rc::new(|ans: &mut ANSDecode| *ans.decode_uniform_from(&LOWERS));
    let fn_d = |ans: &mut ANSDecode| Some(*ans.decode_uniform_from(&DIGITS));
    let fn_m: F1 = Rc::new(|ans: &mut ANSDecode| *ans.decode_uniform_from(&MISC));
    let syms = WeightedSymbols::new(&[
        ClassWeight::new(fn_u, 5),
        ClassWeight::new(fn_l, 5),
        // ClassWeight::new(fn_d, 1),
        ClassWeight::new(fn_m, 1),
    ]);
    let mut etc = Weighted2Stage::new(syms);
    let fn_etc = move |ans: &mut ANSDecode| etc.emit_symbol(ans);

    let rval: SymbolsWithRequirement<char> =
        SymbolsWithRequirement::new(1.0, (5 + 5 + 1) as f32, 1, 12, fn_d, fn_etc);
    rval
}
