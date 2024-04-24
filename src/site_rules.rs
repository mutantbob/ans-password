use crate::asymmetric_numeral_system::{ANSDecode, SimpleClass, Weighted2Stage, WeightedSymbols};
use crate::required_symbols::{ClassWeight, SymbolsWithRequirement};
use crate::symbol_generator::SymbolEmitter;
use crate::{DIGITS, LOWERS, MISC, UPPERS};
use std::rc::Rc;

pub type F1 = Rc<dyn Fn(&mut ANSDecode) -> Option<char>>;

pub fn bob() -> WeightedSymbols<SimpleClass> {
    WeightedSymbols::new(&[
        ClassWeight::new(SimpleClass::Upper, 5),
        ClassWeight::new(SimpleClass::Lower, 5),
        ClassWeight::new(SimpleClass::Digit, 1),
        ClassWeight::new(SimpleClass::Misc, 1),
    ])
}

pub fn bob2() -> Weighted2Stage<char> {
    let fn_u: F1 = Rc::new(|ans: &mut ANSDecode| ans.decode_uniform_from(&UPPERS).copied());
    let fn_l: F1 = Rc::new(|ans: &mut ANSDecode| ans.decode_uniform_from(&LOWERS).copied());
    let fn_d: F1 = Rc::new(|ans: &mut ANSDecode| ans.decode_uniform_from(&DIGITS).copied());
    let fn_m: F1 = Rc::new(|ans: &mut ANSDecode| ans.decode_uniform_from(&MISC).copied());
    let syms = WeightedSymbols::new(&[
        ClassWeight::new(fn_u, 5),
        ClassWeight::new(fn_l, 5),
        ClassWeight::new(fn_d, 1),
        ClassWeight::new(fn_m, 1),
    ]);

    Weighted2Stage::new(syms)
}
pub fn ericsson(symbol_count: u32) -> SymbolsWithRequirement<char> {
    let fn_u: F1 = Rc::new(|ans: &mut ANSDecode| ans.decode_uniform_from(&UPPERS).copied());
    let fn_l: F1 = Rc::new(|ans: &mut ANSDecode| ans.decode_uniform_from(&LOWERS).copied());
    let fn_d = |ans: &mut ANSDecode| ans.decode_uniform_from(&DIGITS).copied();
    let fn_m: F1 = Rc::new(|ans: &mut ANSDecode| ans.decode_uniform_from(&MISC).copied());
    let syms = WeightedSymbols::new(&[
        ClassWeight::new(fn_u, 5),
        ClassWeight::new(fn_l, 5),
        // ClassWeight::new(fn_d, 1),
        ClassWeight::new(fn_m, 1),
    ]);
    let mut etc = Weighted2Stage::new(syms);
    let fn_etc = move |ans: &mut ANSDecode| etc.emit_symbol(ans);

    let rval: SymbolsWithRequirement<char> =
        SymbolsWithRequirement::new(1.0, (5 + 5 + 1) as f32, 1, symbol_count, fn_d, fn_etc);
    rval
}
