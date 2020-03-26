CREATE TYPE indi_func AS ENUM ('Confirm', 'Confirm2', 'Confirm3', 'Baseline', 'Volume', 'Continue', 'Exit');

-- CREATE INDEX indi_in_set_index ON indicator_sets(indicator_id);

CREATE TABLE indicator_sets (
       set_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY
);

CREATE TABLE set_indicators (
       set_id BIGINT REFERENCES indicator_sets(set_id) ON UPDATE CASCADE ON DELETE CASCADE,
       indicator_id INTEGER REFERENCES indicators(indicator_id) ON UPDATE CASCADE, -- TODO delete set_id if indicator_id is delete...
       func indi_func NOT NULL,
       CONSTRAINT set_indi_pk PRIMARY KEY (set_id, indicator_id)
);