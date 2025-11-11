CREATE TABLE IF NOT EXISTS variant_mutations (
    variant_id TEXT NOT NULL,
    gene TEXT NOT NULL,
    position INTEGER NOT NULL,
    reference TEXT NOT NULL,
    mutation TEXT NOT NULL,
    PRIMARY KEY (variant_id, gene, position),
    FOREIGN KEY (variant_id) REFERENCES variants(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_variant_mutations_variant
    ON variant_mutations (variant_id);

