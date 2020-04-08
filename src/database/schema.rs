table! {
    expert_inputs (session_id, input_name) {
        session_id -> Int4,
        input_name -> Varchar,
        input -> Nullable<Numeric>,
        start -> Nullable<Numeric>,
        stop -> Nullable<Numeric>,
        step -> Nullable<Numeric>,
        str_input -> Nullable<Varchar>,
    }
}

table! {
    indicator_inputs (indicator_id, index) {
        indicator_id -> Int4,
        index -> Int2,
        input -> Nullable<Numeric>,
        start -> Nullable<Numeric>,
        stop -> Nullable<Numeric>,
        step -> Nullable<Numeric>,
    }
}

table! {
    indicator_sets (indicator_set_id) {
        indicator_set_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::database::DbIndiFuncMapping;
    indicators (indicator_id) {
        indicator_id -> Int4,
        parent_id -> Nullable<Int4>,
        child_id -> Nullable<Int4>,
        indicator_name -> Varchar,
        shift -> Int2,
        func -> DbIndiFuncMapping,
    }
}

table! {
    results (run_id, indicator_set_id) {
        run_id -> Int8,
        indicator_set_id -> Int8,
        result -> Float8,
        profit -> Float8,
        trades -> Int4,
    }
}

table! {
    run_sessions (session_id) {
        session_id -> Int4,
        start_date -> Date,
        end_date -> Date,
        expert_version -> Nullable<Uuid>,
        symbol_set_id -> Int4,
    }
}

table! {
    runs (run_id) {
        run_id -> Int8,
        session_id -> Int4,
        run_date -> Timestamp,
        indicator_set_id -> Int8,
    }
}

table! {
    set_indicators (indicator_set_id, indicator_id) {
        indicator_set_id -> Int8,
        indicator_id -> Int4,
    }
}

table! {
    set_symbols (symbol_set_id) {
        symbol_set_id -> Int4,
        symbol -> Varchar,
    }
}

table! {
    symbol_sets (symbol_set_id) {
        symbol_set_id -> Int4,
    }
}

joinable!(expert_inputs -> run_sessions (session_id));
joinable!(indicator_inputs -> indicators (indicator_id));
joinable!(results -> indicator_sets (indicator_set_id));
joinable!(results -> runs (run_id));
joinable!(run_sessions -> symbol_sets (symbol_set_id));
joinable!(runs -> indicator_sets (indicator_set_id));
joinable!(runs -> run_sessions (session_id));
joinable!(set_indicators -> indicator_sets (indicator_set_id));
joinable!(set_indicators -> indicators (indicator_id));
joinable!(set_symbols -> symbol_sets (symbol_set_id));

allow_tables_to_appear_in_same_query!(
    expert_inputs,
    indicator_inputs,
    indicator_sets,
    indicators,
    results,
    run_sessions,
    runs,
    set_indicators,
    set_symbols,
    symbol_sets,
);
