CREATE TABLE IF NOT EXISTS variants (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    lineage TEXT NOT NULL,
    location TEXT NOT NULL,
    date TEXT NOT NULL,
    sequence TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS reference_genome (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    sequence TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

