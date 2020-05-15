table! {
    builds (id) {
        id -> Integer,
        created_at -> Timestamp,
        part_id -> Nullable<Integer>,
    }
}

table! {
    inventories (id) {
        id -> Integer,
        quantity -> Integer,
        unit_price -> Float,
        created_at -> Timestamp,
        part_id -> Nullable<Integer>,
    }
}

table! {
    parts (id) {
        id -> Integer,
        pn -> Text,
        mpn -> Text,
        descr -> Text,
        ver -> Integer,
        created_at -> Timestamp,
    }
}

table! {
    parts_parts (id) {
        id -> Integer,
        quantity -> Integer,
        bom_part_id -> Nullable<Integer>,
        part_id -> Nullable<Integer>,
    }
}

joinable!(builds -> parts (part_id));
joinable!(inventories -> parts (part_id));

allow_tables_to_appear_in_same_query!(
    builds,
    inventories,
    parts,
    parts_parts,
);
