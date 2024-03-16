#[derive(Debug, Clone)]
struct ClassWeight<T: Clone + PartialEq> {
    class: T,
    weight: u32,
    minimum: u32,
}

impl<T: Clone + PartialEq> ClassWeight<T> {
    pub fn new(class: T, weight: u32, minimum: u32) -> Self {
        Self {
            class,
            weight,
            minimum,
        }
    }
}

//

pub struct Adjustmotron {
    required_prev: u32,
    optional_prev: u32,
}

impl Adjustmotron {
    pub fn new(required_prev: u32, optional_prev: u32) -> Self {
        Self {
            required_prev,
            optional_prev,
        }
    }

    pub fn restricted_weight(&self, remaining_symbols: u32) -> (u32, u32) {
        if remaining_symbols <= 1 {
            (self.required_prev, 0)
        } else {
            let fewer = remaining_symbols - 1;
            let (r_w, o_w) = self.weight(fewer);
            let w_a = r_w + o_w;
            let (r_w, o_w) = self.restricted_weight(fewer);
            let w_b = r_w + o_w;
            (self.required_prev * w_a, self.optional_prev * w_b)
        }
    }

    pub fn weight(&self, remaining_symbols: u32) -> (u32, u32) {
        let sum = self.required_prev + self.optional_prev;
        let pow = (1..remaining_symbols).into_iter().fold(1, |a, _b| a * sum);
        (self.required_prev * pow, self.optional_prev * pow)
    }
}

/*pub fn adjusted_weights<T: Clone + PartialEq>(
    required: &ClassWeight<T>,
    all: &[ClassWeight<T>],
) -> Vec<ClassWeight<T>> {
    let idx = all.iter().find(|x| x.class == required.class);
    panic!()
}
*/
#[cfg(test)]
mod test {
    use super::Adjustmotron;

    #[test]
    pub fn test1() {
        let atron = Adjustmotron::new(3, 4);
        {
            let (a, b) = atron.weight(1);
            assert_eq!(3, a);
            assert_eq!(4, b);
        }

        {
            let (a, b) = atron.restricted_weight(1);
            assert_eq!(3, a);
            assert_eq!(0, b);
        }
    }

    #[test]
    pub fn test2() {
        {
            let (a, b) = Adjustmotron::new(1, 1).restricted_weight(2);
            assert_eq!(2, a);
            assert_eq!(1, b);
        }

        {
            let (a, b) = Adjustmotron::new(2, 2).restricted_weight(2);
            assert_eq!(2 * 4, a);
            assert_eq!(2 * 2, b);
        }

        {
            let (a, b) = Adjustmotron::new(3, 4).restricted_weight(2);
            assert_eq!(3 * (4 + 3), a);
            assert_eq!(4 * 3, b);
        }
    }

    #[test]
    pub fn test3() {
        {
            let (a, b) = Adjustmotron::new(1, 1).restricted_weight(8);
            assert_eq!(128, a);
            assert_eq!(127, b);
        }
    }
}
