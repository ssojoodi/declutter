CREATE TABLE scans (
    id INTEGER PRIMARY KEY,
    root BLOB NOT NULL,
    depth INTEGER NOT NULL,
    patterns BLOB NOT NULL,
    completed INTEGER NOT NULL,
    UNIQUE(root, depth, patterns)
);

-- IDs follow the scanner's largest-first preorder. 'last' includes descendants.
CREATE TABLE entries (
    scan INTEGER NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
    id INTEGER NOT NULL,
    parent INTEGER NOT NULL,
    name BLOB NOT NULL,
    last INTEGER NOT NULL,
    directory INTEGER NOT NULL,
    real INTEGER NOT NULL,
    temp INTEGER NOT NULL,
    files INTEGER NOT NULL,
    limited INTEGER NOT NULL,
    PRIMARY KEY(scan, id)
) WITHOUT ROWID;
CREATE UNIQUE INDEX entry_by_name ON entries(scan, parent, name);
PRAGMA user_version = 1;
