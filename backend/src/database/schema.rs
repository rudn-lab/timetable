// @generated automatically by Diesel CLI.

diesel::table! {
    faculties (uuid) {
        uuid -> Text,
        name -> Text,
    }
}

diesel::table! {
    groups (uuid) {
        uuid -> Text,
        name -> Text,
        faculty -> Text,
    }
}

diesel::joinable!(groups -> faculties (faculty));

diesel::allow_tables_to_appear_in_same_query!(
    faculties,
    groups,
);
