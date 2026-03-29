#[derive(Debug, Clone)]
pub struct ParsedPlayer {
    pub shirt_number: i32,
    pub first_name: &'static str,
    pub last_name: &'static str,
    pub goals_per_match: &'static [(usize, u32)],
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct ParsedTeam {
    pub name: &'static str,
}

#[derive(Debug, Clone)]
pub struct ParsedMatch {
    pub col_index: usize,
    pub opponent: &'static str,
    pub is_home: bool,
    pub home_score: i32,
    pub away_score: i32,
}

include!(concat!(env!("OUT_DIR"), "/voetbal_data.rs"));
