table! {
    indicator_inputs (indicator_id, index) {
        indicator_id -> Int4,
        index -> Int2,
        input -> Nullable<Float4>,
        start -> Nullable<Float4>,
        stop -> Nullable<Float4>,
        step -> Nullable<Float4>,
    }
}

table! {
    indicators (id) {
        id -> Int4,
        parent_id -> Nullable<Int4>,
        name -> Varchar,
        shift -> Int2,
    }
}

joinable!(indicator_inputs -> indicators (indicator_id));

allow_tables_to_appear_in_same_query!(
    indicator_inputs,
    indicators,
);
