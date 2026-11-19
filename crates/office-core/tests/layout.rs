use office_core::roles::catalog;
use office_core::{CellKind, FloorPlan, Rect};
use std::collections::{HashSet, VecDeque};

const EPS: f32 = 0.01;

fn overlap(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.w - EPS && b.x < a.x + a.w - EPS && a.z < b.z + b.d - EPS && b.z < a.z + a.d - EPS
}

fn on_boundary(p: [f32; 2], r: &Rect) -> bool {
    let inside_x = p[0] >= r.x - EPS && p[0] <= r.x + r.w + EPS;
    let inside_z = p[1] >= r.z - EPS && p[1] <= r.z + r.d + EPS;
    let edge_x = (p[0] - r.x).abs() < EPS || (p[0] - (r.x + r.w)).abs() < EPS;
    let edge_z = (p[1] - r.z).abs() < EPS || (p[1] - (r.z + r.d)).abs() < EPS;
    inside_x && inside_z && (edge_x || edge_z)
}

fn reach(plan: &FloorPlan, from: usize) -> HashSet<usize> {
    let mut seen = HashSet::from([from]);
    let mut q = VecDeque::from([from]);
    while let Some(n) = q.pop_front() {
        for e in &plan.edges {
            let (a, b) = (e[0] as usize, e[1] as usize);
            let next = if a == n { b } else if b == n { a } else { continue };
            if seen.insert(next) {
                q.push_back(next);
            }
        }
    }
    seen
}

#[test]
fn floor_plan_valid() {
    for team in catalog().teams {
        let plan = &team.plan;
        for (i, a) in plan.cells.iter().enumerate() {
            for b in plan.cells.iter().skip(i + 1) {
                assert!(!overlap(&a.rect, &b.rect), "{} overlaps {} on {}", a.key, b.key, team.key);
            }
        }
        assert!(plan.cells.len() >= 6, "{}", team.key);
        for room in &team.rooms {
            assert_eq!(plan.cells.iter().filter(|c| c.key == room.key).count(), 1, "{}", room.key);
        }
        let boss: Vec<_> = plan.cells.iter().filter(|c| c.kind == CellKind::Boss).collect();
        assert_eq!(boss.len(), 1);
        assert!((boss[0].rect.w - 2.0 * boss[0].rect.d).abs() < EPS, "boss is 2 columns wide");
        assert_eq!(plan.cells.iter().filter(|c| c.kind == CellKind::Stairs).count(), 1);
        assert!(plan.cells.iter().any(|c| c.kind == CellKind::Lounge));
        for c in &plan.cells {
            assert!(plan.corridors.iter().any(|r| on_boundary(c.door, r)), "door of {} on {}", c.key, team.key);
            assert_eq!(plan.nodes[c.door_node as usize], c.door);
        }
        let stairs = plan.cells.iter().find(|c| c.kind == CellKind::Stairs).unwrap();
        let seen = reach(plan, stairs.door_node as usize);
        for c in &plan.cells {
            assert!(seen.contains(&(c.door_node as usize)), "{} unreachable on {}", c.key, team.key);
            if matches!(c.kind, CellKind::Role | CellKind::Other) {
                assert_eq!(c.desks.len(), 3, "{}", c.key);
                for d in &c.desks {
                    assert!(d[0] > c.rect.x && d[0] < c.rect.x + c.rect.w, "{}", c.key);
                    assert!(d[1] > c.rect.z && d[1] < c.rect.z + c.rect.d, "{}", c.key);
                }
            }
            assert!(c.standing.len() >= 2);
        }
    }
}

#[test]
fn stairs_aligned() {
    let cat = catalog();
    let rect = |p: &FloorPlan| p.cells.iter().find(|c| c.kind == CellKind::Stairs).unwrap().rect.clone();
    let first = rect(&cat.teams[0].plan);
    for t in &cat.teams {
        assert_eq!(rect(&t.plan), first, "{}", t.key);
    }
}

#[test]
fn software_floor_has_fifteen_cells() {
    let cat = catalog();
    let sw = cat.teams.iter().find(|t| t.key == "software").unwrap();
    assert_eq!(sw.plan.cells.len(), 15);
    assert!((sw.plan.width - 26.5).abs() < EPS && (sw.plan.depth - 31.5).abs() < EPS);
}
