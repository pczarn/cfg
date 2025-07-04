use cfg_load::CfgLoadExt;
use cfg_load::CfgLoadAdvancedExt;
use cfg_grammar::Cfg;

#[test]
fn test_load() {
    let cfg = Cfg::load(r#"
        start ::= a b c d;
        a ::= y;
    "#).unwrap();
    assert_eq!(cfg.rules().count(), 2);
}

#[test]
fn test_load_advanced() {
    let (cfg, _, bs) = Cfg::load_advanced(r#"
        start ::= a b c d;
        a ::= y;
    "#).unwrap();
    assert_eq!(cfg.rules().count(), 2);
    let (cfg, _, bs) = Cfg::load_advanced(r#"
        start ::= a b c d;
        a ::= "y";
    "#).unwrap();
    assert_eq!(cfg.rules().count(), 3);
    let (cfg, _, bs) = Cfg::load_advanced(r#"
        start ::= a b c d;
        a ::= "y";
        lexer {
            x ::= "z";
        }
    "#).unwrap();
    assert_eq!(cfg.rules().count(), 5);
    let (cfg, lm, bs) = Cfg::load_advanced(r#"
        start ::= a b c d;
        a ::= "y";
        lexer {
            x ::= "zzt";
            x ::= Regexp("test");
        }
    "#).unwrap();
    assert_eq!(cfg.rules().count(), 7);
    assert_eq!(format!("{:?}", lm.0.keys().flat_map(|cl| cl.iter().next()).collect::<Vec<_>>()), "[('e', 'f'), ('s', 't'), ('t', 'u'), ('y', 'z'), ('z', '{')]");
    assert_eq!(cfg.to_bnf(), r#"start ::= a b c d;
__lex0 ::= g6;
a ::= __lex0;
__lex1 ::= g9 g9 g10;
x ::= __lex1;
g13 ::= g10 g11 g12 g10;
x ::= g13;
"#);
    assert_eq!(bs.iter().count(), 3);
}

#[test]
fn test_err() {
    assert!(Cfg::load_advanced(r#"
        start ::= a b c d;
        x ::= "y";
        lexer {
            x ::= "zzt";
            x ::= Regexp("test");
        }
    "#).err().is_some());
}

#[test]
fn test_big() {
    let (cfg, lm, bs) = Cfg::load_advanced(r#"
        start ::= a b c d;
        a ::= "y";
        lexer {
            x ::= "zzt";
            x ::= Regexp("test(er)?");
        }
    "#).unwrap();
    assert_eq!(cfg.rules().count(), 11);
    assert_eq!(format!("{:?}", lm.0.keys().flat_map(|cl| cl.iter().next()).collect::<Vec<_>>()), "[('e', 'f'), ('r', 's'), ('s', 't'), ('t', 'u'), ('y', 'z'), ('z', '{')]");
    assert_eq!(cfg.to_bnf(), r#"start ::= a b c d;
__lex0 ::= g6;
a ::= __lex0;
__lex1 ::= g9 g9 g10;
x ::= __lex1;
g14 ::= g11 g12;
g15 ::= ;
g15 ::= g16;
g16 ::= g14;
g17 ::= g10 g11 g13 g10 g15;
x ::= g17;
"#);
    assert_eq!(bs.iter().count(), 3);
}