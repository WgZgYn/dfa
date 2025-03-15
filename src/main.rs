use crate::number::test_number;

mod identity;
mod machine;
mod number;
mod trans_table;

pub trait Sequence {
    type Item;
    fn peek(&mut self) -> Option<&Self::Item>;
    fn next(&mut self) -> Option<Self::Item>;

    // Just for test
    fn next_unwrap(&mut self) -> Self::Item {
        self.next().unwrap()
    }
    fn advance(&mut self);
}

impl<I> Sequence for Box<dyn Sequence<Item = I>> {
    type Item = I;

    fn peek(&mut self) -> Option<&Self::Item> {
        self.as_mut().peek()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.as_mut().next()
    }

    fn advance(&mut self) {
        self.as_mut().advance()
    }
}

trait IntoSequence {
    type Seq: Sequence;
    fn into_sequence(self) -> Self::Seq;
}

trait ParseError {
    fn not_finish() -> Self;
}

impl ParseError for () {
    fn not_finish() -> Self {}
}

trait MatchAll<Seq> {
    type StateType: State;
    type ErrorType: ParseError;
    fn trans_state(
        &mut self,
        curr: Self::StateType,
        input: &mut Seq,
    ) -> Result<Self::StateType, Self::ErrorType>;
}

trait State: PartialEq {
    fn is_final(&self) -> bool;
    fn start() -> Self;
}

mod dfa {
    use crate::{IntoSequence, MatchAll, Sequence, State};
    fn parse<Seq: Sequence, Parser: MatchAll<Seq>>(
        mut seq: Seq,
        mut trans: Parser,
    ) -> Result<Parser::StateType, Parser::ErrorType> {
        let mut curr = Parser::StateType::start();
        while !curr.is_final() {
            curr = trans.trans_state(curr, &mut seq)?
        }
        Ok(curr)
    }

    pub fn parse_from<IntoSeq: IntoSequence, Parser: MatchAll<IntoSeq::Seq>>(
        seq: IntoSeq,
        trans: Parser,
    ) -> Result<Parser::StateType, Parser::ErrorType> {
        parse(seq.into_sequence(), trans)
    }

    pub fn parse_dyn<
        Seq: Sequence + 'static,
        IntoSeq: IntoSequence<Seq = Seq>,
        Parser: MatchAll<Box<dyn Sequence<Item = Seq::Item>>>,
    >(
        seq: IntoSeq,
        mut trans: Parser,
    ) -> Result<Parser::StateType, Parser::ErrorType> {
        let mut seq: Box<dyn Sequence<Item = Seq::Item>> = Box::new(seq.into_sequence());
        let mut curr = Parser::StateType::start();
        while !curr.is_final() {
            curr = trans.trans_state(curr, &mut seq)?
        }
        Ok(curr)
    }
}

fn main() {
    test_number();
    trans_table::test();
}
