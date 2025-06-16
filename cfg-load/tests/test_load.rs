use cfg_load::CfgLoadExt;
use cfg_grammar::Cfg;

#[test]
fn test_load() {
    let cfg = Cfg::load(r#"
        start ::= a b c d;
        a ::= xx;
    "#).unwrap();
    assert_eq!(cfg.rules().count(), 8);
}