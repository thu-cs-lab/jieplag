// @generated automatically by Diesel CLI.

diesel::table! {
    blocks (id) {
        id -> Int4,
        match_id -> Int4,
        left_line_from -> Int4,
        right_line_from -> Int4,
        length -> Int4,
    }
}

diesel::table! {
    matches (id) {
        id -> Int4,
        job_id -> Int4,
        left_submission_id -> Int4,
        left_match_rate -> Int4,
        right_submission_id -> Int4,
        right_match_rate -> Int4,
        lines_matched -> Int4,
    }
}

diesel::table! {
    submissions (id) {
        id -> Int4,
        name -> Text,
        code -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        user_name -> Text,
        salt -> Bytea,
        password -> Bytea,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    blocks,
    matches,
    submissions,
    users,
);
