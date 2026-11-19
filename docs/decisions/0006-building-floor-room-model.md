# 0006. Building, team floor, and room model with handoff events in office-core
- Date: 2026-11-18
- Status: accepted (user-approved 2026-11-18)
- Decision: Session = building and team = fixed floor (1 General, 2 Software, 3 Product, 4 Content, 5 Game). Each known role is a room, and main is a boss office. `office-core` owns `RoleInfo.room`, `SessionView.activeTeams`, `catalog()`, `floor_plan()` (cells, doors, desks, corridor graph), and `Delta.handoffs` (spawn and return, plus a relay heuristic). The UI owns only props, themes, people, and animation.
- Reason: The model stays renderer-agnostic (0003) and unit-testable with `cargo test`. The renderer does not have to infer handoffs from state diffs.
