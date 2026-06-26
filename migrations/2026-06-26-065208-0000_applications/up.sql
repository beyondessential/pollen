-- A configuration record (an artifact) records a user's answers and the
-- content-addressed ruleset they are bound against. Finalising freezes the
-- binding. See spec WIZ (Artifact lifecycle, Content-addressed binding).

CREATE TYPE application_status AS ENUM ('draft', 'finalised');

-- Content-addressed ruleset store: one row per distinct normalized ruleset,
-- keyed by the hash of its canonical content. Identical rulesets dedupe.
CREATE TABLE config_store (
    config_hash text        PRIMARY KEY,
    content     jsonb       NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);

-- An application: the user's answers plus the ruleset hash they're bound to,
-- a draft/finalised status, and lineage back to the version it was forked from.
CREATE TABLE applications (
    id           uuid               PRIMARY KEY DEFAULT gen_random_uuid(),
    answers      jsonb              NOT NULL DEFAULT '{}'::jsonb,
    config_hash  text               NOT NULL REFERENCES config_store (config_hash),
    status       application_status NOT NULL DEFAULT 'draft',
    parent_id    uuid               REFERENCES applications (id),
    created_at   timestamptz        NOT NULL DEFAULT now(),
    finalised_at timestamptz
);

CREATE INDEX applications_config_hash_idx ON applications (config_hash);
CREATE INDEX applications_parent_id_idx ON applications (parent_id);
