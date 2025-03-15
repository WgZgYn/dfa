#[cfg(test)]
mod test {
    use crate::{MatchAll, ParseError, Sequence, State, dfa};

    #[test]
    pub fn test_identity() {
        #[derive(Eq, PartialEq, Hash, Debug)]
        enum IdentityState {
            Start,
            Building,
            Finish(String),
        }

        #[derive(Debug)]
        enum IdentityError {
            IllegalAscii(u8),
            StartWithNumber(u8),
            NotFinish,
            MyFault,
        }

        impl ParseError for IdentityError {
            fn not_finish() -> Self {
                IdentityError::NotFinish
            }
        }

        impl State for IdentityState {
            fn is_final(&self) -> bool {
                if let IdentityState::Finish(_) = self {
                    true
                } else {
                    false
                }
            }

            fn start() -> Self {
                IdentityState::Start
            }
        }

        struct IdentityBuilder {
            buf: Vec<u8>,
        }

        impl IdentityBuilder {
            fn default() -> Self {
                Self { buf: Vec::new() }
            }
        }

        impl<Seq: Sequence<Item = u8>> MatchAll<Seq> for IdentityBuilder {
            type StateType = IdentityState;
            type ErrorType = IdentityError;
            fn trans_state(
                &mut self,
                curr: IdentityState,
                input: &mut Seq,
            ) -> Result<IdentityState, IdentityError> {
                let next = input.peek();
                match (curr, next) {
                    (IdentityState::Start | IdentityState::Building, Some(v))
                        if !v.is_ascii_alphanumeric() =>
                    {
                        Err(IdentityError::IllegalAscii(*v))
                    }
                    (IdentityState::Start, Some(v)) => {
                        if v.is_ascii_digit() {
                            Err(IdentityError::StartWithNumber(*v))
                        } else {
                            self.buf.push(input.next_unwrap());
                            Ok(IdentityState::Building)
                        }
                    }
                    (IdentityState::Building, Some(v)) => {
                        if !v.is_ascii_alphanumeric() {
                            Err(IdentityError::IllegalAscii(*v))
                        } else {
                            self.buf.push(input.next_unwrap());
                            Ok(IdentityState::Building)
                        }
                    }
                    (IdentityState::Building, None) => {
                        let v = std::mem::replace(&mut self.buf, Vec::new());
                        let s = String::from_utf8(v);
                        match s {
                            Ok(s) => Ok(IdentityState::Finish(s)),
                            Err(_) => Err(IdentityError::MyFault),
                        }
                    }
                    _ => Err(IdentityError::NotFinish),
                }
            }
        }

        let ok = dfa::parse_from("metaas123asd".bytes(), IdentityBuilder::default());
        println!("{:?}", ok);

        let ok = dfa::parse_from("met-aas123asd".bytes(), IdentityBuilder::default());
        println!("{:?}", ok);

        let ok = dfa::parse_from("123metaas123asd".bytes(), IdentityBuilder::default());
        println!("{:?}", ok);

        let ok = dfa::parse_from("a1".bytes(), IdentityBuilder::default());
        println!("{:?}", ok);

        let ok = dfa::parse_from("".bytes(), IdentityBuilder::default());
        println!("{:?}", ok);
    }
}
