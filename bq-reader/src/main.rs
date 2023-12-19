#![allow(dead_code)]

use {rust_fsm::*, std::collections::VecDeque};

state_machine! {
    derive(Debug)
    repr_c(true)
    Reader(Backquote)

    // `
        Backquote => {
            At => SyntaxError [ At ],
            Backquote => Backquote,
            Comma => Comma,
            Constant => Form,
            Dot => SyntaxError [ Dot ],
            EndList => Exit,
            List => List,
            Symbol => Quote,
        },

    // `,
        Comma => {
            At => SyntaxError [ At ],
            Backquote => Backquote,
            Comma => Comma,
            Constant => Form,
            Dot => SyntaxError [ Dot ],
            EndList => Exit,
            List => List,
            Symbol => Form,
        },

    // `(
        List => {
            At => SyntaxError [ At ],
            Backquote => Backquote,
            Comma => ListComma,
            Constant => Form,
            Dot => SyntaxError [ Dot ],
            EndList => Exit,
            List => List,
            Symbol => Quote,
        },
    // `,(
        ListComma => {
            At => SyntaxError [ At ],
            Backquote => Backquote,
            Comma => ListComma,
            Constant => Form,
            Dot => SyntaxError [ Dot ],
            EndList => Exit,
            List => List,
            Symbol => Quote,
        },
}

fn main() {
    let mut machine: StateMachine<Reader> = StateMachine::new();

    let mut next_states =
        VecDeque::<ReaderInput>::from_iter(vec![ReaderInput::Comma,
                                                ReaderInput::Constant],
        );

    loop {
        if next_states.is_empty() {
            break;
        }

        let output = machine.consume(&next_states.pop_front().unwrap());
        println!("state {:?}", machine.state());

        match machine.state() {
            ReaderState::Backquote | ReaderState::Form | ReaderState::Exit | ReaderState::Quote => {
                println!("way to go, funbuns")
            }

            ReaderState::SyntaxError => {
                println!("syntax error {:?}", output.unwrap().unwrap());
                break;
            }
            _ => {}
        }
    }
}
