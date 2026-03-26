use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ParsedPlayer {
    pub shirt_number: i32,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone)]
pub struct ParsedTeam {
    pub name: String,
}

pub type ImportError = Box<dyn std::error::Error + Send + Sync>;

pub fn parse_voetbal_csv(path: &str) -> Result<(Vec<ParsedPlayer>, Vec<ParsedTeam>), ImportError> {
    let file = std::fs::File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(file);

    let mut records = rdr.records();

    // Row 0: match headers → extract opponent teams
    let header = records.next().ok_or("CSV is empty")??;
    let teams = extract_teams(&header);

    // Rows 1–8: team stats + empty separator row → skip
    for _ in 0..8 {
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

    Ok((players, teams))
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
    if name_str.is_empty()
        || name_lower == "gastspeler(s)"
        || name_lower == "own goal tegenpartij"
        || name_lower == "totaal"
    {
        return None;
    }

    let shirt_number: i32 = shirt_str.parse().ok()?;
    let (last_name, first_name) = parse_name(name_str)?;

    Some(ParsedPlayer {
        shirt_number,
        first_name,
        last_name,
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
