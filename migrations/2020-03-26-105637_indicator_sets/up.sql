CREATE TYPE indi_func AS ENUM ('confirm', 'confirm2', 'confirm3', 'baseline', 'volume', 'continue', 'exit');

CREATE TABLE indicator_sets (
       indicator_id INTEGER REFERENCES indicators(id) ON DELETE CASCADE,
       indicator_func indi_func NOT NULL,
       PRIMARY KEY (indicator_id, indicator_func)
);
