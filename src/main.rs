trait Sequence {
    type Item;
    fn peek(&self) -> Option<&Self::Item>;
    fn advance(&mut self);
}

trait Trans<I, S: State = usize> {
    fn trans(&self, curr: S, input: &I) -> Option<S>;
}

trait State: Eq + PartialEq + Copy + Clone {}

trait StateSet {
    type Item: State;
    fn contains(&self, state: Self::Item) -> bool;
}

impl State for usize {}
impl State for &'static str {}

impl<S: State> StateSet for &[S] {
    type Item = S;

    fn contains(&self, state: Self::Item) -> bool {
        <[S]>::contains(self, &state)
    }
}

impl<S: State> StateSet for S {
    type Item = S;

    fn contains(&self, state: Self::Item) -> bool {
        self.eq(&state)
    }
}

impl<S: State, const N: usize> StateSet for [S; N] {
    type Item = S;

    fn contains(&self, state: Self::Item) -> bool {
        <[S]>::contains(self, &state)
    }
}

mod dfa {
    use crate::{Sequence, State, StateSet, Trans};

    pub fn parse<I, S: State>(
        start: S,
        stop: impl StateSet<Item = S>,
        mut seq: impl Sequence<Item = I>,
        trans: impl Trans<I, S>,
    ) -> bool {
        let mut curr = start;
        while let Some(inner) = seq.peek() {
            let next = trans.trans(curr, &inner);
            match next {
                None => return stop.contains(curr),
                Some(next) => {
                    curr = next;
                    seq.advance();
                }
            }
        }
        true
    }
}

fn main() {
    let metadata = "asd12a3";
    struct StrSeq<'a> {
        data: &'a [u8],
        ptr: usize,
    }
    impl Sequence for StrSeq<'_> {
        type Item = u8;

        fn peek(&self) -> Option<&Self::Item> {
            self.data.get(self.ptr)
        }

        fn advance(&mut self) {
            self.ptr += 1;
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    enum IdentityState {
        Start,
        StartWithAlpha,
    }

    impl State for IdentityState {}

    struct T;
    impl Trans<u8, IdentityState> for T {
        fn trans(&self, curr: IdentityState, input: &u8) -> Option<IdentityState> {
            if !input.is_ascii_alphanumeric() {
                return None;
            }
            match curr {
                IdentityState::Start => {
                    if input.is_ascii_digit() {
                        None
                    } else {
                        Some(IdentityState::StartWithAlpha)
                    }
                }
                a => Some(a),
            }
        }
    }

    let ok = dfa::parse(
        IdentityState::Start,
        IdentityState::StartWithAlpha,
        StrSeq {
            data: metadata.as_bytes(),
            ptr: 0,
        },
        T,
    );
    println!("Hello, world! {}", ok);
}
