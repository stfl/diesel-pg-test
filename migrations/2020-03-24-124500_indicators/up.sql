CREATE TYPE IndiFunc AS ENUM ('confirm', 'confirm2', 'confirm3', 'baseline', 'volume', 'continue', 'exit');

CREATE TABLE indicators (
       indicator_id INTEGER GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
       parent_id INTEGER REFERENCES indicators(indicator_id) ON UPDATE CASCADE ON DELETE SET NULL,
       child_id INTEGER REFERENCES indicators(indicator_id) ON UPDATE CASCADE ON DELETE SET NULL,
       indicator_name VARCHAR(100) NOT NULL, -- name
       shift SMALLINT NOT NULL DEFAULT 0 CHECK(shift >= 0),
       func IndiFunc NOT NULL
);

CREATE INDEX parents_index ON indicators(parent_id);
CREATE INDEX indi_names_index ON indicators(indicator_name); -- TODO (indicator_name, func)

CREATE TABLE indicator_inputs (
       -- id SERIAL PRIMARY KEY,
       indicator_id INTEGER REFERENCES indicators(indicator_id) ON UPDATE CASCADE ON DELETE CASCADE,
       index SMALLINT NOT NULL CHECK(index >= 0),
       input NUMERIC(12,4),   -- float is not reliable for comparing and restoring values and
       start NUMERIC(12,4),
       stop NUMERIC(12,4),
       step NUMERIC(12,4),
       CONSTRAINT input_notnull CHECK (
       (( input IS NOT NULL OR
          NOT ( start IS NULL OR stop IS NULL or step IS NULL ))
       ) -- either input needs to be set or all of start,stop,step
       ),
       CONSTRAINT indi_index_pk PRIMARY KEY (indicator_id, index)
);

CREATE TABLE indicator_sets (
       indicator_set_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY
);

CREATE TABLE set_indicators (
       indicator_set_id BIGINT REFERENCES indicator_sets(indicator_set_id) ON UPDATE CASCADE ON DELETE CASCADE,
       indicator_id INTEGER REFERENCES indicators(indicator_id) ON UPDATE CASCADE ON DELETE RESTRICT, -- TODO delete set_id if indicator_id is delete...
       CONSTRAINT set_indi_pk PRIMARY KEY (indicator_set_id, indicator_id)
);
