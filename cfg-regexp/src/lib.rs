pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

use regex_syntax::hir::{self, Hir, HirKind};
use regex_syntax::Parser;

pub fn walk_hir(hir: &Hir, depth: usize) {
    let indent = "  ".repeat(depth);

    match hir.kind() {
        HirKind::Literal(lit) => {
            println!("{indent}Literal: {:?}", lit);
        }
        HirKind::Class(class) => {
            println!("{indent}Class: {:?}", class);
        }
        HirKind::Repetition(rep) => {
            println!("{indent}Repetition: {:?}", rep.kind);
            walk_hir(&rep.hir, depth + 1);
        }
        HirKind::Capture(group) => {
            println!("{indent}Group (capturing={}):", group.name.is_some());
            walk_hir(&group.sub, depth + 1);
        }
        HirKind::Concat(exprs) => {
            println!("{indent}Concat:");
            for expr in exprs {
                walk_hir(expr, depth + 1);
            }
        }
        HirKind::Alternation(exprs) => {
            println!("{indent}Alternation:");
            for expr in exprs {
                walk_hir(expr, depth + 1);
            }
        }
        HirKind::Anchor(anchor) => {
            println!("{indent}Anchor: {:?}", anchor);
        }
        HirKind::WordBoundary(boundary) => {
            println!("{indent}WordBoundary: {:?}", boundary);
        }
        HirKind::Empty => {
            println!("{indent}Empty");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use regex_syntax::hir::{self, Hir, HirKind};
    use regex_syntax::Parser;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);


        let pattern = r"(?i)(foo|bar)\d+";
        let hir = Parser::new().parse(pattern).unwrap();
        walk_hir(&hir, 0);
    }
}
