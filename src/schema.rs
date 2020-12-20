table! {
    users (id) {
        id -> Int4,
        user_name -> Text,
        salt -> Bytea,
        password -> Bytea,
    }
}
