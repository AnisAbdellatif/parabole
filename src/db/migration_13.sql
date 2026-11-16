CREATE TABLE trakt_queue (
    id INTEGER PRIMARY KEY,
    profile_id INTEGER NOT NULL REFERENCES profiles ON DELETE CASCADE,
    path TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
