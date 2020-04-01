table! {
    tickets (id) {
        id -> Int4,
        description -> Text,
    }
}

table! {
    tickets_to_user (id) {
        id -> Int4,
        ticket_id -> Nullable<Int4>,
        user_id -> Nullable<Int4>,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
    }
}

joinable!(tickets_to_user -> tickets (ticket_id));
joinable!(tickets_to_user -> users (user_id));

allow_tables_to_appear_in_same_query!(
    tickets,
    tickets_to_user,
    users,
);
