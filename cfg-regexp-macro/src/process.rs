use cfg_load::advanced::AdvancedGrammar;

#[derive(miniserde::Serialize)]
pub struct Info {}

pub fn process(grammar: AdvancedGrammar) -> Info {
    Info {}
}
