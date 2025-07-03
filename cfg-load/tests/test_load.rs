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
    let (cfg, bs) = Cfg::load_advanced(r#"
        start ::= a b c d;
        a ::= y;
    "#).unwrap();
    assert_eq!(cfg.rules().count(), 2);
    let (cfg, bs) = Cfg::load_advanced(r#"
        start ::= a b c d;
        a ::= "y";
    "#).unwrap();
    assert_eq!(cfg.rules().count(), 2);
    let (cfg, bs) = Cfg::load_advanced(r#"
        start ::= a b c d;
        a ::= "y";
        lexer {
            x ::= "z";
        }
    "#).unwrap();
    assert_eq!(cfg.rules().count(), 3);
    let (cfg, bs) = Cfg::load_advanced(r#"
        start ::= a b c d;
        a ::= "y";
        lexer {
            x ::= "z";
            x ::= Regexp("test");
        }
    "#).unwrap();
    assert_eq!(cfg.rules().count(), 3);
}