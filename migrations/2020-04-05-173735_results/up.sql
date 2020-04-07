-- Your SQL goes here

CREATE TABLE symbol_sets (
       symbol_set_id INTEGER GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY
);

CREATE TABLE set_symbols (
       symbol_set_id INTEGER NOT NULL REFERENCES symbol_sets(symbol_set_id) PRIMARY KEY,
       symbol VARCHAR(10) NOT NULL
       -- CONSTRAINT symbol_set_pk PRIMARY KEY trade_symbols_id
);

-- TODO insert all major pairs as index 1

CREATE TABLE run_sessions (
       session_id INTEGER GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
       start_date DATE NOT NULL,
       end_date DATE NOT NULL,
       expert_version UUID,
       symbol_set_id INTEGER NOT NULL REFERENCES symbol_sets(symbol_set_id)
);


CREATE TABLE expert_inputs (
       session_id INTEGER NOT NULL REFERENCES run_sessions(session_id) ON UPDATE CASCADE ON DELETE CASCADE,
       input_name VARCHAR(20) NOT NULL,
       input NUMERIC(12,4),
       start NUMERIC(12,4),
       stop NUMERIC(12,4),
       step NUMERIC(12,4),
       str_input VARCHAR(100),
       CONSTRAINT input_notnull CHECK ((
         input IS NOT NULL
         OR NOT ( start IS NULL OR stop IS NULL or step IS NULL )
         OR str_input IS NOT NULL
       )), -- either input needs to be set or all of start,stop,step
       CONSTRAINT expert_inputs_pk PRIMARY KEY (session_id, input_name)
);

CREATE TABLE runs (
       run_id BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
       session_id INTEGER NOT NULL REFERENCES run_sessions(session_id),
       run_date TIMESTAMP NOT NULL,
       indicator_set_id BIGINT NOT NULL REFERENCES indicator_sets(indicator_set_id) -- most likely ranged
);

CREATE TABLE results (
       run_id BIGINT NOT NULL REFERENCES runs(run_id),
       indicator_set_id BIGINT NOT NULL REFERENCES indicator_sets(indicator_set_id), -- instances of the set defined in run
       run_result FLOAT NOT NULL,
       run_profit FLOAT NOT NULL,
       trades INTEGER NOT NULL CHECK(trades >= 0),
       -- TODO what else can be parsed from the result? -- best if we keep everything?!
       CONSTRAINT results_pk PRIMARY KEY (run_id, indicator_set_id)
);
