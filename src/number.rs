use crate::{IntoSequence, MatchAll, Sequence, State, dfa};
use std::iter::Peekable;

impl<T: Iterator> Sequence for Peekable<T> {
    type Item = T::Item;
    fn peek(&mut self) -> Option<&Self::Item> {
        Peekable::<T>::peek(self)
    }

    fn next(&mut self) -> Option<Self::Item> {
        <Self as Iterator>::next(self)
    }

    fn advance(&mut self) {
        <Self as Iterator>::next(self);
    }
}

impl<T: IntoIterator> IntoSequence for T {
    type Seq = Peekable<T::IntoIter>;
    fn into_sequence(self) -> Self::Seq {
        self.into_iter().peekable()
    }
}

pub fn test_number() {
    struct NumberBuilder {
        p: bool,
        ep: bool,
        a: f64,
        b: f64,
        bi: f64,
        ea: i32,
    }
    impl NumberBuilder {
        fn new() -> Self {
            Self {
                p: true,
                ep: true,
                a: 0.,
                b: 0.,
                bi: 0.1,
                ea: 0,
            }
        }

        fn build(&mut self) -> f64 {
            if !self.p {
                self.a *= -1.;
            }
            if !self.ep {
                self.ea *= -1;
            }
            (self.a + self.b) * 10.0_f64.powi(self.ea)
        }
    }

    #[derive(PartialEq, Debug)]
    enum NumberBuildState {
        Start,   // 0
        Signed,  // 1
        A,       // 2
        Point,   // 3
        B,       // 4
        E,       // 5
        ESigned, // 6
        EA,      // 7
        Finish(f64),
    }

    impl State for NumberBuildState {
        fn is_final(&self) -> bool {
            if let Finish(_) = self { true } else { false }
        }

        fn start() -> Self {
            Start
        }
    }

    impl<Seq: Sequence<Item = u8>> MatchAll<Seq> for NumberBuilder {
        type StateType = NumberBuildState;
        type ErrorType = ();
        fn trans_state(
            &mut self,
            curr: NumberBuildState,
            input: &mut Seq,
        ) -> Result<NumberBuildState, Self::ErrorType> {
            use NumberBuildState::*;
            match curr {
                Start => match input.peek() {
                    Some(s @ (b'+' | b'-')) => {
                        self.p = s == &b'+';
                        input.advance();
                        Ok(Signed)
                    }
                    Some(n @ b'0'..=b'9') => {
                        self.a = self.a * 10. + (n - b'0') as f64;
                        input.advance();
                        Ok(A)
                    }
                    _ => Err(()),
                },
                Signed => match input.peek() {
                    Some(n @ b'0'..=b'9') => {
                        self.a = self.a * 10. + (n - b'0') as f64;
                        input.advance();
                        Ok(A)
                    }
                    _ => Err(()),
                },
                A => match input.peek() {
                    Some(n @ b'0'..=b'9') => {
                        self.a = self.a * 10. + (n - b'0') as f64;
                        input.advance();
                        Ok(A)
                    }
                    Some(b'E' | b'e') => {
                        input.advance();
                        Ok(E)
                    }
                    Some(b'.') => {
                        input.advance();
                        Ok(Point)
                    }
                    None => Ok(Finish(self.build())),
                    _ => Err(()),
                },
                Point => match input.peek() {
                    Some(s @ b'0'..=b'9') => {
                        self.b += self.bi * (s - b'0') as f64;
                        self.bi /= 10.;
                        input.advance();
                        Ok(B)
                    }
                    _ => Err(()),
                },
                B => match input.peek() {
                    Some(s @ b'0'..=b'9') => {
                        self.b += self.bi * (s - b'0') as f64;
                        self.bi /= 10.;
                        input.advance();
                        Ok(B)
                    }
                    Some(b'E' | b'e') => {
                        input.advance();
                        Ok(E)
                    }
                    None => Ok(Finish(self.build())),
                    _ => Err(()),
                },
                E => match input.peek() {
                    Some(s @ b'0'..=b'9') => {
                        self.ea = self.ea * 10 + (s - b'0') as i32;
                        input.advance();
                        Ok(EA)
                    }
                    Some(s @ (b'+' | b'-')) => {
                        self.ep = s == &b'+';
                        input.advance();
                        Ok(ESigned)
                    }
                    _ => Err(()),
                },
                ESigned => match input.peek() {
                    Some(s @ b'0'..=b'9') => {
                        self.ea = self.ea * 10 + (s - b'0') as i32;
                        input.advance();
                        Ok(EA)
                    }
                    _ => Err(()),
                },
                EA => match input.peek() {
                    Some(n @ b'0'..=b'9') => {
                        self.a = self.a * 10. + (n - b'0') as f64;
                        input.advance();
                        Ok(EA)
                    }
                    None => Ok(Finish(self.build())),
                    _ => Err(()),
                },
                _ => match input.peek() {
                    None => Ok(Finish(self.build())),
                    Some(_) => Err(()),
                },
            }
        }
    }

    let s = "+114514.191910E+6";
    let result = dfa::parse_from(s.bytes(), NumberBuilder::new());
    use NumberBuildState::*;
    println!("{:?}", result);
}
