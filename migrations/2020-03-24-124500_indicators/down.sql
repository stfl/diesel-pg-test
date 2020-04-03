-- This file should undo anything in `up.sql`
DROP TABLE indicator_inputs;
-- DROP INDEX indicator_ranged_parent_index;
DROP INDEX parents_index;
DROP INDEX indi_names_index;
DROP TABLE indicator_default_func;
DROP TABLE indicators;
DROP TYPE indi_func;
