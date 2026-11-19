use office_core::roles::role_for;

#[test]
fn role_mapping() {
    let cases = [
        ("gaisser-agents:software-architect", "Architect", "#4F7CAC"),
        ("architect", "Architect", "#4F7CAC"),
        ("gaisser-agents:software-engineer", "Coder", "#5FA35F"),
        ("coder", "Coder", "#5FA35F"),
        ("engineer", "Coder", "#5FA35F"),
        ("gaisser-agents:software-test-engineer", "Tester", "#D9A441"),
        ("tester", "Tester", "#D9A441"),
        ("software-debugger", "Debugger", "#C8553D"),
        ("software-code-reviewer", "Code reviewer", "#8E6BBF"),
        ("software-security-reviewer", "Security", "#B23A48"),
        ("software-performance-reviewer", "Performance", "#E07A2E"),
        ("software-db-engineer", "DB engineer", "#3A8F8F"),
        ("software-devops", "DevOps", "#6C7A89"),
        ("software-diagrammer", "Diagrammer", "#C76FA8"),
        ("software-docs-writer", "Docs", "#9AA36B"),
        ("Explore", "Explorer", "#7FB3D5"),
        ("general-purpose", "General", "#A0A0A0"),
        ("Plan", "Planner", "#6A8EAE"),
    ];
    for (ty, label, color) in cases {
        let r = role_for(Some(ty), false);
        assert_eq!(r.label, label, "{ty}");
        assert_eq!(r.color, color, "{ty}");
    }
    let main = role_for(None, true);
    assert_eq!((main.label.as_str(), main.color.as_str()), ("Main", "#F2C14E"));
    assert_eq!(role_for(None, false).label, "Subagent");
}

#[test]
fn unknown_and_families() {
    let a = role_for(Some("gaisser-agents:foo-bar"), false);
    let b = role_for(Some("gaisser-agents:foo-bar"), false);
    assert_eq!(a.label, "Foo Bar");
    assert_eq!(a.color, b.color);
    assert!(a.color.starts_with('#'));
    let p = role_for(Some("gaisser-agents:product-market-analyst"), false);
    assert_eq!((p.label.as_str(), p.color.as_str(), p.team.as_deref()), ("Market Analyst", "#2A9D8F", Some("product")));
    let c = role_for(Some("content-writer"), false);
    assert_eq!((c.color.as_str(), c.team.as_deref()), ("#E76F9A", Some("content")));
    let g = role_for(Some("game-designer"), false);
    assert_eq!((g.label.as_str(), g.color.as_str()), ("Designer", "#7B5EA7"));
}

#[test]
fn roles_room_and_team() {
    let cases: [(Option<&str>, bool, Option<&str>, &str); 9] = [
        (Some("coder"), false, Some("software"), "software-engineer"),
        (Some("gaisser-agents:software-test-engineer"), false, Some("software"), "software-test-engineer"),
        (Some("Explore"), false, Some("general"), "explore"),
        (Some("Plan"), false, Some("general"), "plan"),
        (Some("general-purpose"), false, Some("general"), "general-purpose"),
        (Some("zzz"), false, Some("general"), "general-other"),
        (Some("product-market-analyst"), false, Some("product"), "product-other"),
        (Some("content-writer"), false, Some("content"), "content-writer"),
        (None, true, None, "boss"),
    ];
    for (ty, main, team, room) in cases {
        let r = role_for(ty, main);
        assert_eq!(r.team.as_deref(), team, "{ty:?}");
        assert_eq!(r.room, room, "{ty:?}");
    }
}

#[test]
fn catalog_complete() {
    let cat = office_core::roles::catalog();
    let keys: Vec<&str> = cat.teams.iter().map(|t| t.key.as_str()).collect();
    assert_eq!(keys, ["general", "software", "product", "content", "game"]);
    let floors: Vec<u8> = cat.teams.iter().map(|t| t.floor).collect();
    assert_eq!(floors, [1, 2, 3, 4, 5]);
    let rooms = |team: &str| -> Vec<String> {
        cat.teams.iter().find(|t| t.key == team).unwrap().rooms.iter().map(|r| r.key.clone()).collect()
    };
    let sw: Vec<String> = [
        "architect", "engineer", "test-engineer", "debugger", "code-reviewer", "security-reviewer",
        "performance-reviewer", "db-engineer", "devops", "diagrammer", "docs-writer",
    ]
    .iter()
    .map(|s| format!("software-{s}"))
    .chain(["software-other".to_string()])
    .collect();
    assert_eq!(rooms("software"), sw);
    let fam = |team: &str, roles: &[&str]| -> Vec<String> {
        roles.iter().map(|r| format!("{team}-{r}")).chain([format!("{team}-other")]).collect()
    };
    assert_eq!(rooms("product"), fam("product", &["discovery", "reviewer", "spec", "ux"]));
    assert_eq!(rooms("content"), fam("content", &["brand", "designer", "qa", "scout", "writer"]));
    assert_eq!(
        rooms("game"),
        fam("game", &["designer", "network-engineer", "programmer", "qa", "technical-artist"])
    );
    assert_eq!(rooms("general"), ["explore", "plan", "general-purpose", "general-other"]);
    // every role_for room exists in its team's catalog
    for ty in ["coder", "Explore", "zzz", "product-spec", "game-qa", "content-x", "software-foo"] {
        let r = role_for(Some(ty), false);
        let team = r.team.clone().unwrap();
        assert!(rooms(&team).contains(&r.room), "{ty} -> {}", r.room);
    }
}
