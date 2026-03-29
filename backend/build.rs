use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let csv_path = PathBuf::from(&manifest_dir).join("../data/voetbal.csv");

    println!("cargo:rerun-if-changed={}", csv_path.display());

    let content = fs::read_to_string(&csv_path).expect("Failed to read ../data/voetbal.csv");

    let (players, teams, matches) = parse_csv(&content);

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join("voetbal_data.rs");

    let mut code = String::new();

    code.push_str("pub static PLAYERS: &[ParsedPlayer] = &[\n");
    for p in &players {
        let goals: String = p
            .goals_per_match
            .iter()
            .map(|(col, g)| format!("({}usize, {}u32)", col, g))
            .collect::<Vec<_>>()
            .join(", ");
        code.push_str(&format!(
            "    ParsedPlayer {{ shirt_number: {}, first_name: \"{}\", last_name: \"{}\", goals_per_match: &[{}], active: {} }},\n",
            p.shirt_number,
            escape(&p.first_name),
            escape(&p.last_name),
            goals,
            p.active,
        ));
    }
    code.push_str("];\n\n");

    code.push_str("pub static TEAMS: &[ParsedTeam] = &[\n");
    for t in &teams {
        code.push_str(&format!(
            "    ParsedTeam {{ name: \"{}\" }},\n",
            escape(&t.name)
        ));
    }
    code.push_str("];\n\n");

    code.push_str("pub static MATCHES: &[ParsedMatch] = &[\n");
    for m in &matches {
        code.push_str(&format!(
            "    ParsedMatch {{ col_index: {}usize, opponent: \"{}\", is_home: {}, home_score: {}, away_score: {} }},\n",
            m.col_index,
            escape(&m.opponent),
            m.is_home,
            m.home_score,
            m.away_score,
        ));
    }
    code.push_str("];\n");

    fs::write(out_path, code).unwrap();
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ---------------------------------------------------------------------------
// Intermediate (owned) types used only during build-time parsing
// ---------------------------------------------------------------------------

struct Player {
    shirt_number: i32,
    first_name: String,
    last_name: String,
    goals_per_match: Vec<(usize, u32)>,
    active: bool,
}

struct Team {
    name: String,
}

struct Match {
    col_index: usize,
    opponent: String,
    is_home: bool,
    home_score: i32,
    away_score: i32,
}

// ---------------------------------------------------------------------------
// Parsing logic (mirrors src/import/mod.rs)
// ---------------------------------------------------------------------------

const NP_ALIASES: &[&str] = &["np", "de noordpool"];

fn parse_csv(content: &str) -> (Vec<Player>, Vec<Team>, Vec<Match>) {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(content.as_bytes());

    let mut records = rdr.records();

    let header = records.next().expect("CSV is empty").unwrap();
    let teams = extract_teams(&header);

    let scores_row = records.next().expect("CSV missing scores row").unwrap();
    let matches = extract_matches(&header, &scores_row);

    for _ in 0..7 {
        records.next();
    }

    let mut players = Vec::new();
    for result in records {
        let record = result.unwrap();
        if let Some(player) = parse_player_row(&record) {
            players.push(player);
        }
    }

    (players, teams, matches)
}

fn extract_teams(record: &csv::StringRecord) -> Vec<Team> {
    const EN_DASH: &str = " \u{2013} ";

    let mut seen = HashSet::new();
    let mut teams = Vec::new();

    for field in record.iter().skip(2) {
        let field = field.trim();
        if field.is_empty() {
            continue;
        }

        let parts: Vec<&str> = field.splitn(2, EN_DASH).collect();
        if parts.len() != 2 {
            continue;
        }

        let left = parts[0].trim();
        let right = parts[1].trim();
        let left_lower = left.to_lowercase();
        let right_lower = right.to_lowercase();

        let opponent = if NP_ALIASES.iter().any(|a| left_lower == *a) {
            strip_paren(right)
        } else if NP_ALIASES.iter().any(|a| right_lower.starts_with(a)) {
            left.to_string()
        } else {
            continue;
        };

        let opponent = opponent.trim().to_string();
        if opponent.is_empty() || NP_ALIASES.iter().any(|a| opponent.to_lowercase() == *a) {
            continue;
        }

        let canonical = canonical_team_name(&opponent);
        let key = dedup_key(&canonical);
        if seen.insert(key) {
            teams.push(Team { name: canonical });
        }
    }

    teams
}

fn extract_matches(header: &csv::StringRecord, scores_row: &csv::StringRecord) -> Vec<Match> {
    const EN_DASH: &str = " \u{2013} ";

    let mut matches = Vec::new();

    for (col_index, field) in header.iter().enumerate().skip(2) {
        let field = field.trim();
        if field.is_empty() {
            continue;
        }

        let parts: Vec<&str> = field.splitn(2, EN_DASH).collect();
        if parts.len() != 2 {
            continue;
        }

        let left = parts[0].trim();
        let right = parts[1].trim();
        let left_lower = left.to_lowercase();
        let right_lower = right.to_lowercase();

        let (opponent, is_home) = if NP_ALIASES.iter().any(|a| left_lower == *a) {
            (strip_paren(right), true)
        } else if NP_ALIASES.iter().any(|a| right_lower.starts_with(a)) {
            (left.to_string(), false)
        } else {
            continue;
        };

        let opponent = opponent.trim().to_string();
        if opponent.is_empty() || NP_ALIASES.iter().any(|a| opponent.to_lowercase() == *a) {
            continue;
        }

        let opponent = canonical_team_name(&opponent);

        let score_str = scores_row.get(col_index).unwrap_or("").trim();
        let (home_score, away_score) = parse_score(score_str).unwrap_or((0, 0));

        matches.push(Match {
            col_index,
            opponent,
            is_home,
            home_score,
            away_score,
        });
    }

    matches
}

fn parse_score(s: &str) -> Option<(i32, i32)> {
    let parts: Vec<&str> = s.splitn(2, '-').collect();
    if parts.len() != 2 {
        return None;
    }
    let home: i32 = parts[0].trim().parse().ok()?;
    let away: i32 = parts[1].trim().parse().ok()?;
    Some((home, away))
}

fn dedup_key(s: &str) -> String {
    let mut key = String::new();
    let mut last_was_sep = false;
    for ch in s.chars() {
        if ch.is_whitespace() || ch == '-' || ch == '\u{2013}' {
            if !last_was_sep && !key.is_empty() {
                key.push(' ');
            }
            last_was_sep = true;
        } else {
            key.push_str(&ch.to_lowercase().to_string());
            last_was_sep = false;
        }
    }
    key
}

fn canonical_team_name(name: &str) -> String {
    match dedup_key(name).as_str() {
        "oud turnhout" => "Oud-Turnhout".to_string(),
        "weelde" | "unitas" => "Unitas".to_string(),
        _ => name.to_string(),
    }
}

fn strip_paren(s: &str) -> String {
    if let Some(idx) = s.find('(') {
        s[..idx].trim().to_string()
    } else {
        s.trim().to_string()
    }
}

fn parse_player_row(record: &csv::StringRecord) -> Option<Player> {
    let shirt_str = record.get(0)?.trim();
    let name_str = record.get(1)?.trim();

    let name_lower = name_str.to_lowercase();
    if name_str.is_empty() || name_lower == "own goal tegenpartij" || name_lower == "totaal" {
        return None;
    }

    let shirt_number: i32 = shirt_str.parse().ok()?;

    let goals_per_match: Vec<(usize, u32)> = record
        .iter()
        .enumerate()
        .skip(2)
        .filter_map(|(col_index, val)| {
            let count: u32 = val.trim().parse().ok()?;
            if count > 0 { Some((col_index, count)) } else { None }
        })
        .collect();

    if name_lower == "gastspeler(s)" {
        return Some(Player {
            shirt_number,
            first_name: "Gastspeler(s)".to_string(),
            last_name: String::new(),
            goals_per_match,
            active: false,
        });
    }

    let (last_name, first_name) = parse_name(name_str)?;

    Some(Player {
        shirt_number,
        first_name,
        last_name,
        goals_per_match,
        active: true,
    })
}

fn parse_name(raw: &str) -> Option<(String, String)> {
    let (last, first) = raw.trim().rsplit_once(' ')?;
    Some((capitalize(last), capitalize(first)))
}

fn capitalize(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
