#![feature(proc_macro_span)]

extern crate proc_macro;

mod process;

use std::iter;

use cfg::Cfg;
use cfg_load::{CfgLoadAdvancedExt, LoadError};
use proc_macro::{Delimiter, Span, TokenStream, TokenTree};
// use std::io::Write;
// use std::process::{Command, Stdio};

fn explicit_delimiters(delim: Delimiter) -> Option<(char, char)> {
    match delim {
        Delimiter::Parenthesis => Some(('(', ')')),
        Delimiter::Brace => Some(('{', '}')),
        Delimiter::Bracket => Some(('[', ']')),
        // FIXME(eddyb) maybe encode implicit delimiters somehow?
        // One way could be to have an opaque `FlatToken` variant,
        // containing the entire group, instead of exposing its contents.
        Delimiter::None => None,
    }
}

fn flatten(stream: TokenStream, out: &mut Vec<(String, Span, bool)>) {
    for tt in stream {
        let flat = match tt {
            TokenTree::Group(tt) => {
                let stream = tt.stream();
                let spans = (tt.span_open(), tt.span_close());
                let delimiters = explicit_delimiters(tt.delimiter());
                if let Some((open, _)) = delimiters {
                    out.push((open.to_string(), spans.0, true));
                }
                flatten(stream, out);
                if let Some((_, close)) = delimiters {
                    (close.to_string(), spans.1, true)
                } else {
                    continue;
                }
            }
            TokenTree::Ident(tt) => (tt.to_string(), tt.span(), true),
            TokenTree::Punct(tt) => {
                let is_colon = tt.as_char() == ':';
                (tt.to_string(), tt.span(), !is_colon)
            }
            TokenTree::Literal(tt) => (tt.to_string(), tt.span(), true),
        };
        out.push(flat);
    }
}

#[proc_macro]
pub fn cfg_regexp(stream: TokenStream) -> TokenStream {
    let mut input = vec![];
    flatten(stream, &mut input);
    let mut code = String::new();
    let mut spans = vec![];
    let mut input_iter = input.into_iter().peekable();
    while let Some((ref s, span, append)) = input_iter.next() {
        code.push_str(s);
        let is_lone_colon = s == ":"
            && Some(span.byte_range().end)
                != input_iter
                    .peek()
                    .map(|&(_, span, _)| span.byte_range().start);
        if append || is_lone_colon {
            spans.push(span);
            code.push('\n');
        }
    }
    match Cfg::load_advanced(&code) {
        Ok(advanced_grammar) => {
            assert!(advanced_grammar.sbs.is_empty());
            let grammar = process::process(advanced_grammar);
            let serialized = miniserde::json::to_string(&grammar);
            format!(
                r##"
                {{
                use panini_runtime::*;
                struct Parser {{ grammar: DefaultGrammar }}
                #[derive(Debug)]
                struct Value;
                impl Parser {{
                    fn new(grammar: &str) -> Self {{
                        Self {{ grammar: json::from_str(grammar).unwrap() }}
                    }}
                    fn parse(&self, s: &str) -> Value {{
                        let mut tokens = vec![];
                        let mut rec: Recognizer<&'_ DefaultGrammar, Bocage> =
                            Recognizer::with_forest(&self.grammar, Bocage::new(&self.grammar));
                        let finished = rec.parse(&tokens[..]).unwrap();
                        assert!(finished);
                        Value
                    }}
                }}
                Parser::new(r#"{}"#) }}
            "##,
                serialized
            )
            .parse()
            .unwrap()
        }
        Err(load_error) => match load_error {
            LoadError {
                reason,
                line,
                col: _,
                token: _,
            } => {
                let result: TokenStream = format!(r##"compile_error!("{}")"##, reason)
                    .parse()
                    .unwrap();
                let mut result2 = TokenStream::new();
                for mut token in result.into_iter() {
                    if line != 0 {
                        token.set_span(spans[line as usize - 1]);
                    }
                    result2.extend(iter::once(token));
                }
                result2
            }
        },
    }
    // let mut input = vec![];
    // flatten(stream, &mut input);
    // let mut lua = Command::new("lua5.4")
    //     .arg("../panini.lua")
    //     .stdin(Stdio::piped())
    //     .stdout(Stdio::piped())
    //     .spawn()
    //     .unwrap();
    // if let Some(mut stdin) = lua.stdin.take() {
    //     // Write a string into stdin
    //     stdin.write_all(b"Hello from Rust!\n").unwrap();
    // }
    // let lua_output = lua.wait_with_output().unwrap();

    // match lua_output.status.code() {
    //     Some(0) => String::from_utf8_lossy(&lua_output.stdout).parse().unwrap(),
    //     Some(code) => panic!("Error {}", code),
    //     None => panic!("None")
    // }
}
