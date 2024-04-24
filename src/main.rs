use crate::asymmetric_numeral_system::{ANSDecode, SimpleClass, WeightedSymbols};
use crate::symbol_generator::SymbolEmitter;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use sha1::{Digest, Sha1};
use std::io::{BufRead, Stdin};
use std::ops::Deref;

mod asymmetric_numeral_system;
mod required_symbols;
mod site_rules;
mod symbol_generator;

const UPPERS: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const LOWERS: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const MISC: [char; 32] = [
    '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';', '<', '=',
    '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~',
];

struct Password {
    base: Vec<u8>,
}

impl Password {
    pub fn from_site_and_password(site: &str, password: &str) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(site.as_bytes());
        hasher.update(password.as_bytes());
        hasher.update("\n".as_bytes()); // compatibility with original perl app

        let result = hasher.finalize().deref().to_vec();

        Self { base: result }
    }

    pub fn base64_short(&self) -> String {
        let result64 = BASE64_STANDARD.encode(&self.base);
        let first12 = result64.chars().take(12);
        String::from_iter(first12)
    }

    pub fn via_ans<'a: 'b, 'b, SE>(&self, emitter: &'a mut SE) -> String
    where
        SE: SymbolEmitter<'b, char>,
    {
        println!("digest has {} bytes", self.base.len());
        let mut ans = ANSDecode::new(self.base.iter().copied());

        let e2: *mut SE = emitter as *mut _; // this is some shenanigans to outsmart the borrow checker until I can figure out https://stackoverflow.com/questions/78379390/explicit-lifetime-for-self-in-trait-seems-to-cause-e0499-cannot-borrow-emitte

        let mut rval = String::new();
        for _ in 0..30 {
            let ch = unsafe { &mut *e2 }.emit_symbol(&mut ans);
            rval.push(ch);
        }
        rval
    }
}

pub fn weighted_password_symbols(
    ans: &mut ANSDecode,
    weighted_symbols: &WeightedSymbols<SimpleClass>,
) -> char {
    let class = ans.decode_from_weights(weighted_symbols);
    let symbol_subset: &[char] = match class {
        SimpleClass::Upper => &UPPERS,
        SimpleClass::Lower => &LOWERS,
        SimpleClass::Digit => &DIGITS,
        SimpleClass::Misc => &MISC,
    };
    *ans.decode_uniform_from(symbol_subset)
}

pub struct StdinLineFetcher {
    pub stdin: Stdin,
}

impl StdinLineFetcher {
    pub fn new() -> Self {
        Self {
            stdin: std::io::stdin(),
        }
    }

    pub fn read_from_stdin_if_missing(&mut self, val: Option<String>) -> String {
        match val {
            None => {
                let stdin = std::io::stdin();
                let mut lines = stdin.lock().lines();
                let line1 = lines.next();
                line1.unwrap().unwrap()
            }
            Some(val) => val,
        }
    }
}

impl Default for StdinLineFetcher {
    fn default() -> Self {
        Self::new()
    }
}

//

fn main() {
    let (site, password) = fetch_site_and_key();

    display_pins(&site, &password);
}

fn fetch_site_and_key() -> (String, String) {
    let args = std::env::args();
    let mut args_iter = args.into_iter();
    let _cmd = args_iter.next();
    let site = args_iter.next();
    let pin = if false {
        let mut stdin = StdinLineFetcher::new();
        stdin.read_from_stdin_if_missing(args_iter.next())
    } else {
        match args_iter.next() {
            None => rpassword::prompt_password("secret: ").expect("failed to read secret from tty"),
            Some(val) => val,
        }
    };

    println!("site = {:?}", site);
    println!("pin = {:?}", pin);

    let site = site.unwrap();
    let password = pin;
    (site, password)
}

fn display_pins(site: &str, password: &str) {
    let result = Password::from_site_and_password(site, password);

    println!("result {:x?}", result.base);

    let result64 = result.base64_short();
    println!("base64 = {}", result64);

    // let mut weighted_symbols = WeightedSymbols::<()>::bob2();
    let mut weighted_symbols = site_rules::ericsson();

    let result_ans = result.via_ans(&mut weighted_symbols);
    println!("ANS = {}", result_ans);
}

#[cfg(test)]
mod test {
    #[test]
    pub fn bootstrap_symbols() {
        let mut upper_letters = String::new();
        let mut lower_letters = String::new();
        let mut digits = String::new();
        let mut misc = String::new();
        for i in 33..127 {
            let ch = char::from_u32(i).unwrap();
            let class = if ch.is_ascii_uppercase() {
                &mut upper_letters
            } else if ch.is_ascii_lowercase() {
                &mut lower_letters
            } else if ch.is_ascii_digit() {
                &mut digits
            } else {
                &mut misc
            };
            class.push(ch);
        }

        println!(
            "const UPPERS:[char;{}] = [ {} ];",
            upper_letters.len(),
            as_vector_payload(&upper_letters)
        );
        println!(
            "const LOWERS:[char;{}] = [ {} ];",
            lower_letters.len(),
            as_vector_payload(&lower_letters)
        );
        println!(
            "const DIGITS:[char;{}] = [ {} ];",
            digits.len(),
            as_vector_payload(&digits)
        );
        println!(
            "const MISC:[char;{}] = [ {} ];",
            misc.len(),
            as_vector_payload(&misc)
        );
    }

    fn as_vector_payload(str: &str) -> String {
        String::from_iter(str.chars().map(|ch| match ch {
            '\'' | '\\' => format!("'\\{}', ", ch),
            _ => format!("'{}', ", ch),
        }))
    }
}
