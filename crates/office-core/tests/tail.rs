use office_core::tail::{StartAt, TailReader};
use std::fs::OpenOptions;
use std::io::Write;

fn append(path: &std::path::Path, data: &[u8]) {
    let mut f = OpenOptions::new().create(true).append(true).open(path).unwrap();
    f.write_all(data).unwrap();
}

#[test]
fn tail_partial_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.jsonl");
    std::fs::write(&path, b"").unwrap();
    let mut r = TailReader::new(&path, StartAt::Beginning);
    // write 1: line 1 plus the first half of line 2
    append(&path, b"{\"type\":\"user\"}\n{\"type\":\"assis");
    let l = r.read_new().expect("no error");
    assert_eq!(l.len(), 1);
    assert_eq!(l[0], b"{\"type\":\"user\"}");
    // write 2: more of line 2, still mid-JSON
    append(&path, b"tant\",\"x\":");
    assert_eq!(r.read_new().expect("no error").len(), 0);
    // write 3: the rest of line 2
    append(&path, b"1}\n");
    let l = r.read_new().expect("no error");
    assert_eq!(l.len(), 1);
    assert_eq!(l[0], b"{\"type\":\"assistant\",\"x\":1}");
}

#[test]
fn tail_truncation_resets() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.jsonl");
    std::fs::write(&path, b"{\"a\":1}\n{\"a\":2}\n{\"a\":3}\n").unwrap();
    let mut r = TailReader::new(&path, StartAt::Beginning);
    assert_eq!(r.read_new().unwrap().len(), 3);
    assert!(r.offset > 0);
    // truncate below the offset and write fresh content
    std::fs::write(&path, b"{\"b\":1}\n").unwrap();
    let l = r.read_new().expect("no error after truncation");
    assert_eq!(l, vec![b"{\"b\":1}".to_vec()]);
    // deleting the file returns Ok(empty) and flags it
    std::fs::remove_file(&path).unwrap();
    assert_eq!(r.read_new().unwrap(), Vec::<Vec<u8>>::new());
    assert!(r.gone);
}

#[test]
fn last_bytes_skips_the_cut_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.jsonl");
    std::fs::write(&path, b"{\"a\":1}\n{\"a\":2}\n{\"a\":3}\n").unwrap();
    // starts inside line 2: only line 3 is complete and valid
    let mut r = TailReader::new(&path, StartAt::LastBytes(11));
    assert!(r.truncated_start);
    let l = r.read_new().unwrap();
    assert_eq!(l, vec![b"{\"a\":3}".to_vec()]);
}

#[test]
fn crlf_is_stripped_and_empty_lines_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.jsonl");
    std::fs::write(&path, b"{\"a\":1}\r\n\n{\"a\":2}\n").unwrap();
    let mut r = TailReader::new(&path, StartAt::Beginning);
    let l = r.read_new().unwrap();
    assert_eq!(l, vec![b"{\"a\":1}".to_vec(), b"{\"a\":2}".to_vec()]);
}
