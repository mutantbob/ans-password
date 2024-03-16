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
/*
struct Requirement<T: Clone + PartialEq> {
    requirement: Vec<ClassWeight<T>>,
}

impl<T: Clone + PartialEq> Requirement<T> {
    pub fn new(requirement: impl Into<Vec<ClassWeight<T>>>) -> Self {
        Self {
            requirement: requirement.into(),
        }
    }

    pub(crate) fn adjusted_weights(&self, remaining: i32) -> Vec<ClassWeight<T>> {
        Vec::from_iter(self.requirement.iter().cloned())
    }
}
*/
//

pub struct Adjustmotron {}
impl Adjustmotron {
    pub fn restricted_weight(
        remaining_symbols: u32,
        required_prev: u32,
        optional_prev: u32,
    ) -> (u32, u32) {
        if remaining_symbols <= 1 {
            (required_prev, 0)
        } else {
            let fewer = remaining_symbols - 1;
            let (r_w, o_w) = Self::weight(fewer, required_prev, optional_prev);
            let w_a = r_w + o_w;
            let (r_w, o_w) = Self::restricted_weight(fewer, required_prev, optional_prev);
            let w_b = r_w + o_w;
            (required_prev * w_a, optional_prev * w_b)
        }
    }

    pub fn weight(remaining_symbols: u32, required_prev: u32, optional_prev: u32) -> (u32, u32) {
        let sum = required_prev + optional_prev;
        let pow = (1..remaining_symbols).into_iter().fold(1, |a, _b| a * sum);
        (required_prev * pow, optional_prev * pow)
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
    pub fn test3() {
        {
            let (a, b) = Adjustmotron::weight(1, 3, 4);
            assert_eq!(3, a);
            assert_eq!(4, b);
        }

        {
            let (a, b) = Adjustmotron::restricted_weight(1, 3, 4);
            assert_eq!(3, a);
            assert_eq!(0, b);
        }
    }

    #[test]
    pub fn test4() {
        {
            let (a, b) = Adjustmotron::restricted_weight(2, 1, 1);
            assert_eq!(2, a);
            assert_eq!(1, b);
        }

        {
            let (a, b) = Adjustmotron::restricted_weight(2, 2, 2);
            assert_eq!(2 * 4, a);
            assert_eq!(2 * 2, b);
        }

        {
            let (a, b) = Adjustmotron::restricted_weight(2, 3, 4);
            assert_eq!(3 * (4 + 3), a);
            assert_eq!(4 * 3, b);
        }
    }
}
