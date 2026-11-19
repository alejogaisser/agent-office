//! Floor plans: a grid of 6x6 rooms with corridors, doors, desk slots and a walk graph.
//! Pure data (ADR 0006); the renderer only draws it.
use crate::model::{Cell, CellKind, FloorPlan, Rect, RoomSpec};
use std::collections::HashMap;

pub const CELL: f32 = 6.0;
pub const CORRIDOR: f32 = 2.5;
const COLS: usize = 4;
const PITCH: f32 = CELL + CORRIDOR;

struct Slot {
    row: usize,
    col: usize,
    w: f32,
    key: String,
    label: String,
    kind: CellKind,
}

fn slot(row: usize, col: usize, w: f32, key: &str, label: &str, kind: CellKind) -> Slot {
    Slot { row, col, w, key: key.into(), label: label.into(), kind }
}

/// Position inside a cell given in "door faces south" coordinates, mirrored for doors on the north.
fn place(rect: &Rect, lx: f32, lz: f32, door_south: bool) -> (f32, f32) {
    if door_south {
        (rect.x + lx, rect.z + lz)
    } else {
        (rect.x + rect.w - lx, rect.z + rect.d - lz)
    }
}

pub fn floor_plan(_team: &str, rooms: &[RoomSpec]) -> FloorPlan {
    let mut slots: Vec<Slot> = vec![
        slot(0, 0, 2.0 * CELL, "boss", "Boss", CellKind::Boss),
        slot(0, 3, CELL, "stairs", "Stairs", CellKind::Stairs),
    ];
    // free slots in fill order: row 0 col 2, then rows 1.. all columns
    let mut free: Vec<(usize, usize)> = vec![(0, 2)];
    let mut row = 1;
    let mut placed = 0;
    let mut last_row = 0;
    loop {
        for col in 0..COLS {
            free.push((row, col));
        }
        row += 1;
        if free.len() >= rooms.len() {
            break;
        }
    }
    for (i, room) in rooms.iter().enumerate() {
        let (r, c) = free[i];
        let kind = if i + 1 == rooms.len() { CellKind::Other } else { CellKind::Role };
        slots.push(slot(r, c, CELL, &room.key, &room.label, kind));
        placed = i + 1;
        last_row = r;
    }
    // lounges fill the rest of the last row; a full last row gets a new row of two lounges
    let mut lounge = 0;
    let mut rest: Vec<(usize, usize)> = free[placed..].iter().copied().filter(|(r, _)| *r == last_row).collect();
    if rest.is_empty() {
        last_row += 1;
        rest = vec![(last_row, 0), (last_row, 1)];
    }
    for (r, c) in rest {
        lounge += 1;
        slots.push(slot(r, c, CELL, &format!("lounge-{lounge}"), "Lounge", CellKind::Lounge));
    }
    let rows = last_row + 1;
    let width = COLS as f32 * CELL + CORRIDOR;
    let depth = rows as f32 * CELL + (rows - 1) as f32 * CORRIDOR;

    let corridors_n = rows - 1;
    let mut corridors: Vec<Rect> = (0..corridors_n)
        .map(|i| Rect { x: 0.0, z: i as f32 * PITCH + CELL, w: COLS as f32 * CELL, d: CORRIDOR })
        .collect();
    corridors.push(Rect { x: COLS as f32 * CELL, z: 0.0, w: CORRIDOR, d: depth });
    let center_z = |i: usize| i as f32 * PITCH + CELL + CORRIDOR / 2.0;

    let mut nodes: Vec<[f32; 2]> = Vec::new();
    let mut edges: Vec<[u16; 2]> = Vec::new();
    let mut cells: Vec<Cell> = Vec::new();
    // (corridor, door x) -> projection node
    let mut proj: HashMap<(usize, i32), u16> = HashMap::new();
    let mut per_corridor: Vec<Vec<(f32, u16)>> = vec![Vec::new(); corridors_n];

    slots.sort_by_key(|s| (s.row, s.col));
    for s in &slots {
        let rect = Rect { x: s.col as f32 * CELL, z: s.row as f32 * PITCH, w: s.w, d: CELL };
        let door_south = s.row + 1 != rows || rows == 1;
        let door = if door_south {
            [rect.x + rect.w / 2.0, rect.z + rect.d]
        } else {
            [rect.x + rect.w / 2.0, rect.z]
        };
        let corridor = if door_south { s.row } else { s.row - 1 };
        let door_node = nodes.len() as u16;
        nodes.push(door);
        let key = (corridor, (door[0] * 100.0).round() as i32);
        let p = *proj.entry(key).or_insert_with(|| {
            let id = nodes.len() as u16;
            nodes.push([door[0], center_z(corridor)]);
            per_corridor[corridor].push((door[0], id));
            id
        });
        edges.push([door_node, p]);

        let rot = if door_south { 0.0 } else { std::f32::consts::PI };
        let (desks, standing) = match s.kind {
            CellKind::Role | CellKind::Other => {
                let d = [1.2f32, 3.0, 4.8]
                    .iter()
                    .map(|lx| {
                        let (x, z) = place(&rect, *lx, 2.3, door_south);
                        [x, z, rot]
                    })
                    .collect();
                let st = [1.5f32, 4.5]
                    .iter()
                    .map(|lx| {
                        let (x, z) = place(&rect, *lx, 4.4, door_south);
                        [x, z]
                    })
                    .collect();
                (d, st)
            }
            CellKind::Boss => {
                let (x, z) = place(&rect, 6.0, 2.3, door_south);
                let st = [3.0f32, 9.0]
                    .iter()
                    .map(|lx| {
                        let (x, z) = place(&rect, *lx, 4.4, door_south);
                        [x, z]
                    })
                    .collect();
                (vec![[x, z, rot]], st)
            }
            _ => {
                let st = [1.5f32, 4.5]
                    .iter()
                    .map(|lx| {
                        let (x, z) = place(&rect, *lx, 3.0, door_south);
                        [x, z]
                    })
                    .collect();
                (Vec::new(), st)
            }
        };
        cells.push(Cell {
            key: s.key.clone(),
            label: s.label.clone(),
            kind: s.kind,
            rect,
            door,
            door_node,
            desks,
            standing,
        });
    }

    // corridor spines: projections chained by x, last one joins the connector junction
    let mut junctions: Vec<u16> = Vec::new();
    for i in 0..corridors_n {
        let jid = nodes.len() as u16;
        nodes.push([COLS as f32 * CELL + CORRIDOR / 2.0, center_z(i)]);
        junctions.push(jid);
        let mut list = per_corridor[i].clone();
        list.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for w in list.windows(2) {
            edges.push([w[0].1, w[1].1]);
        }
        if let Some(last) = list.last() {
            edges.push([last.1, jid]);
        }
    }
    for w in junctions.windows(2) {
        edges.push([w[0], w[1]]);
    }
    FloorPlan { width, depth, cells, corridors, nodes, edges }
}
