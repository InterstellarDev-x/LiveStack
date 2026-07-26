// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "website_status"))]
    pub struct WebsiteStatus;
}

diesel::table! {
    channel_link (id) {
        id -> Text,
        channel -> Text,
        channel_user_id -> Text,
        user_id -> Nullable<Text>,
        pairing_code -> Text,
        history -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    incident (id) {
        id -> Text,
        website_id -> Text,
        started_at -> Timestamp,
        resolved_at -> Nullable<Timestamp>,
        cause -> Text,
    }
}

diesel::table! {
    region (id) {
        id -> Text,
        name -> Text,
    }
}

diesel::table! {
    status_page (id) {
        id -> Text,
        user_id -> Text,
        slug -> Text,
        title -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    status_page_monitor (id) {
        id -> Text,
        status_page_id -> Text,
        website_id -> Text,
        display_name -> Text,
        sort_order -> Int4,
    }
}

diesel::table! {
    user (id) {
        id -> Text,
        username -> Text,
        password -> Text,
        email -> Nullable<Text>,
        email_alerts_enabled -> Bool,
    }
}

diesel::table! {
    website (id) {
        id -> Text,
        url -> Text,
        time_added -> Timestamp,
        user_id -> Text,
    }
}

diesel::table! {
    website_notification_config (website_id) {
        website_id -> Text,
        webhook_url -> Nullable<Text>,
        webhook_secret -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        webhook_enabled -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::WebsiteStatus;

    website_tick (id) {
        id -> Text,
        response_time_ms -> Int4,
        status -> WebsiteStatus,
        region_id -> Text,
        website_id -> Text,
        createdAt -> Timestamp,
        dns_time_ms -> Int4,
        connection_time_ms -> Int4,
        tls_time_ms -> Int4,
        data_transfer_time_ms -> Int4,
        waiting_time_ms -> Int4,
    }
}

diesel::joinable!(channel_link -> user (user_id));
diesel::joinable!(incident -> website (website_id));
diesel::joinable!(status_page -> user (user_id));
diesel::joinable!(status_page_monitor -> status_page (status_page_id));
diesel::joinable!(status_page_monitor -> website (website_id));
diesel::joinable!(website -> user (user_id));
diesel::joinable!(website_notification_config -> website (website_id));
diesel::joinable!(website_tick -> region (region_id));
diesel::joinable!(website_tick -> website (website_id));

diesel::allow_tables_to_appear_in_same_query!(
    channel_link,
    incident,
    region,
    status_page,
    status_page_monitor,
    user,
    website,
    website_notification_config,
    website_tick,
);
