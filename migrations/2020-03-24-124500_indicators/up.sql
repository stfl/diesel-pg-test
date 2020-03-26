CREATE TABLE indicators (
       id SERIAL PRIMARY KEY,
       parent_id INTEGER REFERENCES indicators(id),
       name VARCHAR(100) NOT NULL,
       shift SMALLINT NOT NULL DEFAULT 0
);

CREATE INDEX parents_index ON indicators(parent_id);

CREATE TABLE indicator_inputs (
       -- id SERIAL PRIMARY KEY,
       indicator_id INTEGER REFERENCES indicators(id) ON DELETE CASCADE,
       index SMALLINT NOT NULL CHECK(index >= 0),
       input FLOAT4,
       start FLOAT4,
       stop FLOAT4,
       step FLOAT4
       CONSTRAINT input_notnull CHECK (
       (( input IS NOT NULL OR
         NOT ( start IS NULL OR stop IS NULL or step IS NULL ))
       ) -- either input needs to be set or all of start,stop,step
       ),
       PRIMARY KEY (indicator_id, index)
);
