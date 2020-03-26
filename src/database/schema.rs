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
    indicator_sets (set_id) {
        set_id -> Int8,
    }
}

table! {
    indicators (indicator_id) {
        indicator_id -> Int4,
        parent_id -> Nullable<Int4>,
        indicator -> Varchar,
        shift -> Int2,
    }
}

joinable!(indicator_inputs -> indicators (indicator_id));

allow_tables_to_appear_in_same_query!(
    indicator_inputs,
    indicator_sets,
    indicators,
);
