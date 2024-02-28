CREATE TABLE IF NOT EXISTS trash_entry (
    id INTEGER PRIMARY KEY,
    expiration INTEGER NOT NULL CHECK (expiration > created_at),
    abspath TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    blake3sum TEXT NOT NULL,
    mtime INTEGER NOT NULL,
    atime INTEGER NOT NULL,
    unix_mode INTEGER NOT NULL,
    uid INTEGER NOT NULL,
    gid INTEGER NOT NULL,
    created_at INTEGER DEFAULT (unixepoch()),
    UNIQUE (abspath),
    UNIQUE (expiration)
);
