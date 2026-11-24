use crate::config::Config;
use crate::models::{DirEntry, FileType};
use anyhow::{ensure, Result};
use rusqlite::{params, Connection, OptionalExtension, Rows, Statement};
use std::ffi::OsStr;
use std::fs::DirBuilder;
use std::os::unix::{ffi::OsStrExt, fs::DirBuilderExt};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MINUTE: i64 = 60_000;
const MAX_AGE: i64 = 15 * MINUTE;

pub(super) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

pub(super) fn directory() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Caches"));
    #[cfg(not(target_os = "macos"))]
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")));
    base.map(|base| base.join("declutter"))
}

pub(super) fn prepare_directory(path: &Path) -> std::io::Result<PathBuf> {
    DirBuilder::new().recursive(true).mode(0o700).create(path)?;
    path.canonicalize()
}

struct SnapshotMatch {
    completed: i64,
    scan: i64,
    entry: i64,
}

fn find_entry(db: &Connection, scan: i64, relative: &Path) -> Result<Option<i64>> {
    let mut id = 1;
    let mut children =
        db.prepare_cached("SELECT id FROM entries WHERE scan=? AND parent=? AND name=?")?;
    for name in relative.iter() {
        let child = children
            .query_row(params![scan, id, name.as_bytes()], |r| r.get(0))
            .optional()?;
        let Some(child) = child else { return Ok(None) };
        id = child;
    }
    Ok(Some(id))
}

pub(super) struct Cache {
    db: Connection,
    patterns: Vec<u8>,
    depth: usize,
}

impl Cache {
    pub(super) fn open(directory: &Path, config: &Config) -> Result<Self> {
        let mut db = Connection::open(directory.join("trees.sqlite3"))?;
        db.pragma_update(None, "foreign_keys", true)?;
        let version =
            |db: &Connection| db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0));
        if version(&db)? == 0 {
            let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            if version(&tx)? == 0 {
                tx.execute_batch(include_str!("cache.sql"))?;
            }
            tx.commit()?;
        }
        ensure!(version(&db)? == 1, "unsupported tree cache version");
        let mut patterns = Vec::new();
        for pattern in &config.temp_patterns {
            patterns.extend_from_slice(&(pattern.len() as u64).to_le_bytes());
            patterns.extend_from_slice(pattern.as_bytes());
        }
        Ok(Self {
            db,
            patterns,
            depth: config.performance.max_depth,
        })
    }

    pub(super) fn load(&mut self, root: &Path, now: i64) -> Result<Option<DirEntry>> {
        let tx = self.db.transaction()?;
        let mut best: Option<SnapshotMatch> = None;
        for ancestor in root.ancestors() {
            let scan = tx
                .query_row(
                    "SELECT id, completed FROM scans WHERE root=? AND depth=? AND patterns=?
                 AND completed > ? AND completed <= ?",
                    params![
                        ancestor.as_os_str().as_bytes(),
                        self.depth,
                        self.patterns,
                        now - MAX_AGE,
                        now
                    ],
                    |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
                )
                .optional()?;
            let Some((scan, completed)) = scan else {
                continue;
            };
            if best.as_ref().is_some_and(|m| m.completed >= completed) {
                continue;
            }
            let Some(id) = find_entry(&tx, scan, root.strip_prefix(ancestor)?)? else {
                continue;
            };
            let (count, limited): (i64, bool) = tx.query_row(
                "SELECT files, limited FROM entries WHERE scan=? AND id=?",
                [scan, id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            let ttl = if count > 1000 { MAX_AGE } else { MINUTE };
            if now - completed < ttl && (id == 1 || !limited) {
                best = Some(SnapshotMatch {
                    completed,
                    scan,
                    entry: id,
                });
            }
        }
        let Some(selected) = best else {
            return Ok(None);
        };
        // Preorder IDs place an entire subtree in one contiguous range.
        let mut stmt = tx.prepare(
            "SELECT id, last, name, directory, real, temp FROM entries
            WHERE scan=?1 AND id BETWEEN ?2 AND (SELECT last FROM entries WHERE scan=?1 AND id=?2)
            ORDER BY id",
        )?;
        let mut rows = stmt.query([selected.scan, selected.entry])?;
        Ok(Some(read_node(&mut rows, root, true, selected.entry)?.0))
    }

    pub(super) fn store(&mut self, tree: &DirEntry, completed: i64) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let key = params![
            tree.path.as_os_str().as_bytes(),
            self.depth,
            self.patterns,
            completed
        ];
        // An older concurrent scan must not replace a newer snapshot.
        let newer: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM scans WHERE root=? AND depth=?
            AND patterns=? AND completed>?)",
            key,
            |r| r.get(0),
        )?;
        if newer {
            return Ok(());
        }
        tx.execute(
            "DELETE FROM scans WHERE completed <= ?",
            [completed - MAX_AGE],
        )?;
        tx.execute(
            "DELETE FROM scans WHERE root=? AND depth=? AND patterns=?",
            params![tree.path.as_os_str().as_bytes(), self.depth, self.patterns],
        )?;
        tx.execute(
            "INSERT INTO scans(root, depth, patterns, completed) VALUES(?,?,?,?)",
            key,
        )?;
        {
            let mut stmt = tx.prepare("INSERT INTO entries
                (scan,id,parent,name,last,directory,real,temp,files,limited) VALUES(?,?,?,?,?,?,?,?,?,?)")?;
            write_node(&mut stmt, tx.last_insert_rowid(), tree, 0, 1, self.depth)?;
        }
        tx.commit()?;
        Ok(())
    }
}

struct Subtree {
    last: i64,
    files: usize,
    limited: bool,
}

fn write_node(
    stmt: &mut Statement,
    scan: i64,
    node: &DirEntry,
    parent: i64,
    id: i64,
    depth_left: usize,
) -> Result<Subtree> {
    let mut subtree = Subtree {
        last: id,
        files: usize::from(!node.is_dir),
        limited: node.is_dir && depth_left == 0,
    };
    for child in node.children.iter().flatten() {
        let child = write_node(
            stmt,
            scan,
            child,
            id,
            subtree.last + 1,
            depth_left.saturating_sub(1),
        )?;
        subtree.last = child.last;
        subtree.files += child.files;
        subtree.limited |= child.limited;
    }
    stmt.execute(params![
        scan,
        id,
        parent,
        node.path.file_name().unwrap_or_default().as_bytes(),
        subtree.last,
        node.is_dir,
        node.real_size,
        node.temp_size,
        subtree.files,
        subtree.limited
    ])?;
    Ok(subtree)
}

fn read_node(rows: &mut Rows, parent: &Path, root: bool, id: i64) -> Result<(DirEntry, i64)> {
    let row = rows.next()?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    let last: i64 = row.get(1)?;
    ensure!(
        row.get::<_, i64>(0)? == id && last >= id,
        "invalid subtree range"
    );
    let path = if root {
        parent.to_owned()
    } else {
        let name = OsStr::from_bytes(row.get_ref(2)?.as_blob()?);
        ensure!(
            Path::new(name).components().next() == Some(Component::Normal(name)),
            "invalid cached filename"
        );
        parent.join(name)
    };
    let mut node = DirEntry::new(path, row.get(3)?);
    node.real_size = row.get(4)?;
    node.temp_size = row.get(5)?;
    node.total_size = node
        .real_size
        .checked_add(node.temp_size)
        .ok_or_else(|| anyhow::anyhow!("invalid cached size"))?;
    node.file_type = FileType::from_sizes(node.real_size, node.temp_size);
    let mut cursor = id;
    if let Some(children) = &mut node.children {
        while cursor < last {
            let (child, end) = read_node(rows, &node.path, false, cursor + 1)?;
            cursor = end;
            children.push(child);
        }
    }
    ensure!(cursor == last, "invalid cached children");
    Ok((node, last))
}

#[cfg(test)]
#[path = "../../tests/unit/cache.rs"]
mod tests;
