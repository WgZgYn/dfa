use crate::{dfa, IntoSequence, MatchAll, ParseError, Sequence, State};
use std::collections::HashMap;
use std::hash::Hash;

struct TransMachine<Seq, Context> {
    seq: Seq,
    context: Context,
}

impl<Seq, Context> TransMachine<Seq, Context> {
    fn from<IntoSeq: IntoSequence<Seq = Seq>>(seq: IntoSeq, context: Context) -> Self {
        Self {
            seq: seq.into_sequence(),
            context,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum Hashed {
    Start, // 0
    A,
    AA,
    AAB,
    AABB,
}

impl State for Hashed {
    fn is_final(&self) -> bool {
        matches!(self, Hashed::AABB)
    }

    fn start() -> Self {
        Self::Start
    }
}

pub trait Handler {
    type SequenceType: Sequence;
    type StateType: State;
    type ErrorType: ParseError;
    fn filter(
        &self,
        curr: &Self::StateType,
        next: Option<&<Self::SequenceType as Sequence>::Item>,
    ) -> bool;
    fn handle(
        &mut self,
        curr: Self::StateType,
        input: &mut Self::SequenceType,
    ) -> Result<Self::StateType, Self::ErrorType>;
}

type DynHandler<S, Seq, E> = Box<dyn Handler<ErrorType = E, SequenceType = Seq, StateType = S>>;

pub struct TransTable<S, Seq, E> {
    table: HashMap<S, Vec<DynHandler<S, Seq, E>>>,
}

impl<S, Seq, E> TransTable<S, Seq, E> {
    fn new() -> TransTable<S, Seq, E> {
        Self {
            table: HashMap::new(),
        }
    }
    fn register<H: Handler<ErrorType = E, SequenceType = Seq, StateType = S> + 'static>(
        &mut self,
        s: S,
        f: H,
    ) -> &mut Self
    where
        S: Eq + Hash,
        Seq: Sequence,
    {
        self.table
            .entry(s)
            .or_insert_with(Vec::new)
            .push(Box::new(f));
        self
    }
}

impl<S: State + Eq + Hash, Seq: Sequence, E: ParseError> MatchAll<Seq>
    for TransTable<S, Seq, E>
{
    type StateType = S;
    type ErrorType = E;
    fn trans_state(&mut self, curr: S, input: &mut Seq) -> Result<S, Self::ErrorType> {
        match self.table.get_mut(&curr) {
            Some(v) => {
                let f = v.into_iter().find(|v| v.filter(&curr, input.peek()));
                if let Some(f) = f {
                    f.handle(curr, input)
                } else {
                    Err(Self::ErrorType::not_finish())
                }
            }
            None => {
                if curr.is_final() {
                    Ok(curr)
                } else {
                    Err(E::not_finish())
                }
            }
        }
    }
}

pub mod handler {
    use crate::{ParseError, Sequence, State};
    use std::marker::PhantomData;

    pub struct WithFilter<I, P> {
        filter: P,
        phantom: PhantomData<fn(I) -> bool>,
    }
    pub struct ClosureHandler<I, State, Error, Pred, Func> {
        handle: Func,
        filter: Option<Pred>,
        _phantom: PhantomData<(I, State, Error)>,
    }

    pub fn filter<I, P: for<'a> Fn(Option<&'a I>) -> bool>(filter: P) -> WithFilter<I, P> {
        WithFilter {
            filter,
            phantom: PhantomData,
        }
    }

    pub fn to<
        I,
        S: State,
        E: ParseError,
        F: Fn(&mut Box<dyn Sequence<Item = I>>) -> Result<S, E>,
    >(
        trans: F,
    ) -> ClosureHandler<I, S, E, (), F> {
        ClosureHandler {
            handle: trans,
            filter: None,
            _phantom: PhantomData,
        }
    }

    impl<I, P: for<'a> Fn(Option<&'a I>) -> bool> WithFilter<I, P> {
        pub fn to<
            S: State,
            E: ParseError,
            F: Fn(&mut Box<dyn Sequence<Item = I>>) -> Result<S, E>,
        >(
            self,
            trans: F,
        ) -> ClosureHandler<I, S, E, P, F> {
            ClosureHandler {
                handle: trans,
                filter: Some(self.filter),
                _phantom: PhantomData,
            }
        }
    }
    impl<
        I,
        S: State,
        E: ParseError,
        P: for<'a> Fn(Option<&'a I>) -> bool,
        F: for<'a> Fn(&mut Box<dyn Sequence<Item = I>>) -> Result<S, E>,
    > crate::trans_table::Handler for ClosureHandler<I, S, E, P, F>
    {
        type SequenceType = Box<dyn Sequence<Item = I>>;
        type StateType = S;
        type ErrorType = E;

        fn filter(
            &self,
            _curr: &Self::StateType,
            input: Option<&<Self::SequenceType as Sequence>::Item>,
        ) -> bool {
            let f = &self.filter;
            if let Some(f) = f { f(input) } else { true }
        }

        fn handle(
            &mut self,
            _curr: Self::StateType,
            input: &mut Box<dyn Sequence<Item = I>>,
        ) -> Result<Self::StateType, Self::ErrorType> {
            let f = &self.handle;
            f(input)
        }
    }

    impl<
        I,
        S: State,
        E: ParseError,
        F: for<'a> Fn(&mut Box<dyn Sequence<Item = I>>) -> Result<S, E>,
    > crate::trans_table::Handler for ClosureHandler<I, S, E, (), F>
    {
        type SequenceType = Box<dyn Sequence<Item = I>>;
        type StateType = S;
        type ErrorType = E;

        fn filter(
            &self,
            _curr: &Self::StateType,
            _input: Option<&<Self::SequenceType as Sequence>::Item>,
        ) -> bool {
            true
        }

        fn handle(
            &mut self,
            _curr: Self::StateType,
            input: &mut Box<dyn Sequence<Item = I>>,
        ) -> Result<Self::StateType, Self::ErrorType> {
            let f = &self.handle;
            f(input)
        }
    }
}

pub fn test() {
    let mut tb = TransTable::new();

    tb.register(
        Hashed::Start,
        handler::filter(|i| {
            if let Some(b'a' | b'b') = i {
                true
            } else {
                false
            }
        })
        .to(|i| {
            i.advance();
            Ok(Hashed::A)
        }),
    )
    .register(
        Hashed::A,
        handler::filter(|i| i.is_some()).to(|i| match i.peek() {
            Some(b'a' | b'A') => {
                i.advance();
                Ok(Hashed::AA)
            }
            _ => Err(()),
        }),
    )
    .register(
        Hashed::AA,
        handler::filter(|i| i.is_some()).to(|i| match i.peek() {
            Some(b'b' | b'B') => {
                i.advance();
                Ok(Hashed::AAB)
            }
            _ => Err(()),
        }),
    )
    .register(
        Hashed::AAB,
        handler::filter(|i| i.is_some()).to(|i| match i.peek() {
            Some(b'b' | b'B') => {
                i.advance();
                Ok(Hashed::AABB)
            }
            _ => Err(()),
        }),
    )
    .register(
        Hashed::AABB,
        handler::to(|i| match i.peek() {
            None => Ok(Hashed::AABB),
            _ => Err(()),
        }),
    );

    let res = dfa::parse_dyn("aabb".bytes(), tb);
    println!("{:?}", res);
}
