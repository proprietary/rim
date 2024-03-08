CREATE TABLE IF NOT EXISTS trash_entry (
    id INTEGER PRIMARY KEY,
    created_at INTEGER DEFAULT (unixepoch()),
    expiration INTEGER NOT NULL CHECK (expiration > created_at),
    blake3sum TEXT NOT NULL,
    original_path TEXT NOT NULL,
    trash_path TEXT NOT NULL,
    is_dir BOOL NOT NULL DEFAULT FALSE,
    is_link BOOL GENERATED ALWAYS AS (link_target IS NOT NULL) VIRTUAL,
    link_target TEXT DEFAULT NULL,
    file_size INTEGER NOT NULL,
    mtime INTEGER NOT NULL,
    atime INTEGER NOT NULL,
    unix_mode INTEGER NOT NULL,
    uid INTEGER NOT NULL,
    gid INTEGER NOT NULL,
    UNIQUE (blake3sum),
    UNIQUE (original_path),
    UNIQUE (expiration)
);

CREATE INDEX file_hash_slug_idx ON trash_entry(substr(blake3sum, 1, 7));
