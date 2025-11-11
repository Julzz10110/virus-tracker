CREATE TABLE IF NOT EXISTS epidemiology_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collected_at TEXT NOT NULL,
    total_variants INTEGER NOT NULL,
    growth_rate REAL,
    reproduction_number REAL,
    doubling_time REAL
);

