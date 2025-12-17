-- Create sequences table for number generation
CREATE TABLE sequences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    prefix TEXT NOT NULL,
    next_value INTEGER NOT NULL DEFAULT 1,
    digits INTEGER NOT NULL DEFAULT 2,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(prefix)
);

-- Create index for faster lookups
CREATE INDEX idx_sequences_prefix ON sequences(prefix);