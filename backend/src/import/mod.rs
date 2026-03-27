use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ParsedPlayer {
    pub shirt_number: i32,
    pub first_name: String,
    pub last_name: String,
    pub goals_per_match: Vec<(usize, u32)>, // (col_index, goal_count), only goals > 0
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct ParsedTeam {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ParsedMatch {
    pub col_index: usize,
    pub opponent: String,
    pub is_home: bool,
    pub home_score: i32, // left team in header (home team)
    pub away_score: i32, // right team in header (away team)
}

pub type ImportError = Box<dyn std::error::Error + Send + Sync>;

pub fn parse_voetbal_csv(
    path: &str,
) -> Result<(Vec<ParsedPlayer>, Vec<ParsedTeam>, Vec<ParsedMatch>), ImportError> {
    let file = std::fs::File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(file);

    let mut records = rdr.records();

    // Row 0: match headers → extract opponent teams and match info
    let header = records.next().ok_or("CSV is empty")??;
    let teams = extract_teams(&header);

    // Row 1: scores (e.g. "1-0", "2-3")
    let scores_row = records.next().ok_or("CSV missing scores row")??;
    let matches = extract_matches(&header, &scores_row);

    // Rows 2–8: team stats + empty separator row → skip (7 more rows)
    for _ in 0..7 {
        records.next();
    }

    // Rows 9+: player data
    let mut players = Vec::new();
    for result in records {
        let record = result?;
        if let Some(player) = parse_player_row(&record) {
            players.push(player);
        }
    }

    Ok((players, teams, matches))
}

// Team names that refer to De Noordpool itself
const NP_ALIASES: &[&str] = &["np", "de noordpool"];

fn extract_teams(record: &csv::StringRecord) -> Vec<ParsedTeam> {
    // En-dash separator used in match headers
    const EN_DASH: &str = " \u{2013} ";

    let mut seen = HashSet::new();
    let mut teams = Vec::new();

    // Skip col 0 (empty/BOM) and col 1 ("UITSLAGEN ..." label)
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
            // NP is on the left → opponent is on the right
            strip_paren(right)
        } else if NP_ALIASES.iter().any(|a| right_lower.starts_with(a)) {
            // NP is on the right → opponent is on the left
            left.to_string()
        } else {
            continue;
        };

        let opponent = opponent.trim().to_string();
        // Skip if the opponent name itself is an NP alias
        if opponent.is_empty() || NP_ALIASES.iter().any(|a| opponent.to_lowercase() == *a) {
            continue;
        }

        let canonical = canonical_team_name(&opponent);
        let key = dedup_key(&canonical);
        if seen.insert(key) {
            teams.push(ParsedTeam { name: canonical });
        }
    }

    teams
}

fn extract_matches(header: &csv::StringRecord, scores_row: &csv::StringRecord) -> Vec<ParsedMatch> {
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

        matches.push(ParsedMatch {
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

/// Deduplication key: lowercase, all hyphens/en-dashes/whitespace collapsed to a single space.
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

/// Maps known spelling variants to a single canonical display name.
fn canonical_team_name(name: &str) -> String {
    match dedup_key(name).as_str() {
        "oud turnhout" => "Oud-Turnhout".to_string(),
        "weelde" | "unitas" => "Unitas".to_string(),
        _ => name.to_string(),
    }
}

fn strip_paren(s: &str) -> String {
    // Strip parenthetical suffixes like " (oud-T" or "(unitas)"
    if let Some(idx) = s.find('(') {
        s[..idx].trim().to_string()
    } else {
        s.trim().to_string()
    }
}

fn parse_player_row(record: &csv::StringRecord) -> Option<ParsedPlayer> {
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
            if count > 0 {
                Some((col_index, count))
            } else {
                None
            }
        })
        .collect();

    if name_lower == "gastspeler(s)" {
        return Some(ParsedPlayer {
            shirt_number,
            first_name: "Gastspeler(s)".to_string(),
            last_name: "".to_string(),
            goals_per_match,
            active: false,
        });
    }

    let (last_name, first_name) = parse_name(name_str)?;

    Some(ParsedPlayer {
        shirt_number,
        first_name,
        last_name,
        goals_per_match,
        active: true,
    })
}

fn parse_name(raw: &str) -> Option<(String, String)> {
    // Format is "lastname firstname" — split at the last space
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
