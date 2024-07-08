use cfg_symbol::Symbol;

#[test]
fn test_repr() {
    assert_eq!(::std::mem::size_of::<Symbol>(), 4);
}

#[test]
fn test_repr_option_optimization() {
    assert_eq!(::std::mem::size_of::<Option<Symbol>>(), 4);
}
