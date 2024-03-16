use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use sha1::{Digest, Sha1};
use std::io::{BufRead, Stdin};
use std::ops::Deref;

mod required_symbols;

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

fn main() {
    let (site, password) = fetch_site_and_key();

    display_pins(&site, &password);
}

fn fetch_site_and_key() -> (String, String) {
    let mut stdin = StdinLineFetcher::new();
    let args = std::env::args();
    let mut args_iter = args.into_iter();
    let _cmd = args_iter.next();
    let site = args_iter.next();
    let pin = stdin.read_from_stdin_if_missing(args_iter.next());

    println!("site = {:?}", site);
    println!("pin = {:?}", pin);

    let site = site.unwrap();
    let password = pin;
    (site, password)
}

fn display_pins(site: &String, password: &String) {
    let result = Password::from_site_and_password(&site, &password);

    println!("result {:x?}", result.base);

    let result64 = result.base64_short();
    println!("baes64 = {}", result64);
}
