use cfg_grammar::Cfg;
use regex_syntax::Parser;

use crate::{LexerMap, Translator};

pub trait CfgRegexpExt: Sized {
    fn from_regexp(regexp: &str) -> Result<(Self, LexerMap), regex_syntax::Error>;
}

impl CfgRegexpExt for Cfg {
    fn from_regexp(regexp: &str) -> Result<(Self, LexerMap), regex_syntax::Error> {
        Parser::new().parse(regexp).map(Translator::cfg_from_hir)
    }
}
