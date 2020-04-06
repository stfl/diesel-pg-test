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
    indicators (indicator_id) {
        indicator_id -> Int4,
        parent_id -> Nullable<Int4>,
        refined_id -> Nullable<Int4>,
        indicator_name -> Varchar,
        shift -> Int2,
        func -> Indifunc,
    }
}

table! {
    results (run_id, indicator_set_id) {
        run_id -> Int8,
        indicator_set_id -> Int8,
        run_result -> Float8,
        run_profit -> Float8,
        trades -> Int4,
    }
}

table! {
    run_sessions (session_id) {
        session_id -> Int4,
        start_date -> Date,
        end_date -> Date,
        expert_version -> Nullable<Uuid>,
        symbols_set_id -> Int4,
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
    session_symbol_sets (symbols_set_id) {
        symbols_set_id -> Int4,
    }
}

table! {
    session_symbols (symbols_set_id) {
        symbols_set_id -> Int4,
        symbol -> Varchar,
    }
}

joinable!(expert_inputs -> run_sessions (session_id));
joinable!(indicator_inputs -> indicators (indicator_id));
joinable!(results -> indicator_sets (indicator_set_id));
joinable!(results -> runs (run_id));
joinable!(run_sessions -> session_symbol_sets (symbols_set_id));
joinable!(runs -> indicator_sets (indicator_set_id));
joinable!(runs -> run_sessions (session_id));
joinable!(session_symbols -> session_symbol_sets (symbols_set_id));

allow_tables_to_appear_in_same_query!(
    expert_inputs,
    indicator_inputs,
    indicator_sets,
    indicators,
    results,
    run_sessions,
    runs,
    session_symbol_sets,
    session_symbols,
);
