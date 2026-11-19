//! Agent type -> role label and color.
use crate::layout::floor_plan;
use crate::model::{Catalog, RoleInfo, RoomSpec, TeamSpec};
use std::sync::OnceLock;

const TABLE: &[(&[&str], &str, &str, Option<&str>)] = &[
    (&["software-architect", "architect"], "Architect", "#4F7CAC", Some("software")),
    (&["software-engineer", "coder", "engineer"], "Coder", "#5FA35F", Some("software")),
    (&["software-test-engineer", "tester"], "Tester", "#D9A441", Some("software")),
    (&["software-debugger"], "Debugger", "#C8553D", Some("software")),
    (&["software-code-reviewer"], "Code reviewer", "#8E6BBF", Some("software")),
    (&["software-security-reviewer"], "Security", "#B23A48", Some("software")),
    (&["software-performance-reviewer"], "Performance", "#E07A2E", Some("software")),
    (&["software-db-engineer"], "DB engineer", "#3A8F8F", Some("software")),
    (&["software-devops"], "DevOps", "#6C7A89", Some("software")),
    (&["software-diagrammer"], "Diagrammer", "#C76FA8", Some("software")),
    (&["software-docs-writer"], "Docs", "#9AA36B", Some("software")),
    (&["Explore"], "Explorer", "#7FB3D5", Some("general")),
    (&["Plan"], "Planner", "#6A8EAE", Some("general")),
    (&["general-purpose"], "General", "#A0A0A0", Some("general")),
];

/// Roles with their own room, per family (the suffix after the prefix).
const KNOWN_FAMILY_ROLES: &[(&str, &[&str])] = &[
    ("product", &["discovery", "reviewer", "spec", "ux"]),
    ("content", &["brand", "designer", "qa", "scout", "writer"]),
    ("game", &["designer", "network-engineer", "programmer", "qa", "technical-artist"]),
];

/// Teams in floor order (floor = index + 1): key, label.
const TEAMS: &[(&str, &str)] = &[
    ("general", "General"),
    ("software", "Software"),
    ("product", "Product"),
    ("content", "Content"),
    ("game", "Game"),
];

const OTHER_COLOR: &str = "#8A8F98";

const FAMILIES: &[(&str, &str, &str)] = &[
    ("product-", "product", "#2A9D8F"),
    ("content-", "content", "#E76F9A"),
    ("game-", "game", "#7B5EA7"),
];

const PALETTE: [&str; 8] = [
    "#E5989B", "#B5838D", "#6D9DC5", "#80B918", "#F4A259", "#8E7DBE", "#5BC0BE", "#C9ADA7",
];

fn fnv1a(s: &str) -> u32 {
    let mut h: u32 = 0x811c9dc5;
    for b in s.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x01000193);
    }
    h
}

pub fn title_case(s: &str) -> String {
    s.split(|c| c == '-' || c == '_' || c == ' ')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut cs = w.chars();
            match cs.next() {
                Some(f) => f.to_uppercase().collect::<String>() + cs.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn role_for(agent_type: Option<&str>, is_main: bool) -> RoleInfo {
    if is_main {
        return RoleInfo {
            key: "main".into(),
            label: "Main".into(),
            color: "#F2C14E".into(),
            team: None,
            room: "boss".into(),
        };
    }
    let raw = agent_type.unwrap_or("").trim();
    let key = raw.rsplit(':').next().unwrap_or("");
    if key.is_empty() {
        return RoleInfo {
            key: "subagent".into(),
            label: "Subagent".into(),
            color: PALETTE[0].into(),
            team: Some("general".into()),
            room: "general-other".into(),
        };
    }
    for (keys, label, color, team) in TABLE {
        if keys.contains(&key) {
            return RoleInfo {
                key: key.to_string(),
                label: (*label).into(),
                color: (*color).into(),
                team: team.map(str::to_string),
                room: keys[0].to_lowercase(),
            };
        }
    }
    for (prefix, team, color) in FAMILIES {
        if let Some(suffix) = key.strip_prefix(prefix) {
            let known = KNOWN_FAMILY_ROLES
                .iter()
                .any(|(t, roles)| t == team && roles.contains(&suffix));
            return RoleInfo {
                room: if known { key.to_string() } else { format!("{team}-other") },
                key: key.to_string(),
                label: if suffix.is_empty() { title_case(team) } else { title_case(suffix) },
                color: (*color).into(),
                team: Some((*team).into()),
            };
        }
    }
    let (team, room) = if key.starts_with("software-") {
        ("software", "software-other")
    } else {
        ("general", "general-other")
    };
    RoleInfo {
        key: key.to_string(),
        label: title_case(key),
        color: PALETTE[(fnv1a(key) % 8) as usize].into(),
        team: Some(team.into()),
        room: room.into(),
    }
}

fn room_label(key: &str, team: &str) -> String {
    let suffix = key.strip_prefix(&format!("{team}-")).unwrap_or(key);
    match suffix {
        "qa" => "QA".into(),
        "ux" => "UX".into(),
        s => title_case(s),
    }
}

fn team_rooms(team: &str) -> Vec<RoomSpec> {
    let mut rooms: Vec<RoomSpec> = Vec::new();
    for (keys, label, color, t) in TABLE {
        if *t == Some(team) {
            rooms.push(RoomSpec {
                key: keys[0].to_lowercase(),
                label: (*label).into(),
                color: (*color).into(),
            });
        }
    }
    let family_color = FAMILIES.iter().find(|(_, t, _)| *t == team).map(|f| f.2).unwrap_or(OTHER_COLOR);
    if let Some((_, roles)) = KNOWN_FAMILY_ROLES.iter().find(|(t, _)| *t == team) {
        for s in roles.iter() {
            let key = format!("{team}-{s}");
            rooms.push(RoomSpec { label: room_label(&key, team), key, color: family_color.into() });
        }
    }
    rooms.push(RoomSpec { key: format!("{team}-other"), label: "Other".into(), color: family_color.into() });
    rooms
}

/// The fixed building: five team floors with their rooms and floor plans.
pub fn catalog() -> Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG
        .get_or_init(|| Catalog {
            teams: TEAMS
                .iter()
                .enumerate()
                .map(|(i, (key, label))| {
                    let rooms = team_rooms(key);
                    TeamSpec {
                        key: (*key).into(),
                        label: (*label).into(),
                        floor: (i + 1) as u8,
                        plan: floor_plan(key, &rooms),
                        rooms,
                    }
                })
                .collect(),
        })
        .clone()
}

/// Floor index (0-based) of a team key; unknown teams sort last.
pub fn team_order(team: &str) -> usize {
    TEAMS.iter().position(|(k, _)| *k == team).unwrap_or(TEAMS.len())
}
