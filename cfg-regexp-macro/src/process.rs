use cfg_load::advanced::AdvancedGrammar;

#[derive(miniserde::Serialize)]
pub struct Info {}

pub fn process(_grammar: AdvancedGrammar) -> Info {
    Info {}
}
