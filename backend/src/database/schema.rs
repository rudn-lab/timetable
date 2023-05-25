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

diesel::table! {
    timetables (id) {
        id -> Integer,
        name -> Text,
        day -> Text,
        start_time -> Text,
        end_time -> Text,
        student_group -> Text,
    }
}

diesel::joinable!(groups -> faculties (faculty));
diesel::joinable!(timetables -> groups (student_group));

diesel::allow_tables_to_appear_in_same_query!(
    faculties,
    groups,
    timetables,
);
