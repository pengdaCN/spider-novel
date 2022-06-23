CREATE TABLE IF NOT EXISTS novels
(
    id                      INTEGER NOT NULL,
    name                    TEXT    NOT NULL,
    cover                   TEXT,
    author                  TEXT    NOT NULL,
    last_updated_section    TEXT    NOT NULL,
    last_updated_section_at INTEGER,
    last_graped_at          INTEGER NOT NULL,
    PRIMARY KEY (id)
);