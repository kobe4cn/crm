-- Add migration script here
/*create table user_stats(
 email varchar(64) NOT NULL PRIMARY KEY,
 name varchar(64) NOT NULL,
 created_at timestamptz DEFAULT CURRENT_TIMESTAMP,
 last_visited_at timestamptz,
 last_watched_at timestamptz,
 recent_watched int[],
 viewed_but_not_started int[],
 started_but_not_finished int[],
 finished int[],
 last_email_notification timestamptz,
 last_in_app_notification timestamptz,
 last_sms_notification timestamptz,
 );*/
create type gender as enum('male', 'female', 'unknown');
create table if NOT EXISTS user_stats(
  email varchar(128) NOT NULL PRIMARY KEY,
  name varchar(64) NOT NULL,
  gender gender DEFAULT 'unknown',
  created_at timestamptz DEFAULT CURRENT_TIMESTAMP,
  last_visited_at timestamptz,
  last_watched_at timestamptz,
  recent_watched int [],
  viewed_but_not_started int [],
  started_but_not_finished int [],
  finished int [],
  last_email_notification timestamptz,
  last_in_app_notification timestamptz,
  last_sms_notification timestamptz
);
CREATE index if NOT EXISTS user_stats_email_idx ON user_stats(created_at);
CREATE index if NOT EXISTS user_stats_name_idx ON user_stats(last_visited_at);
CREATE index if NOT EXISTS user_stats_last_watched_at_idx ON user_stats(last_watched_at);
CREATE index if NOT EXISTS user_stats_recent_watched_idx ON user_stats using GIN(recent_watched);
CREATE index if NOT EXISTS user_stats_viewed_but_not_started_idx ON user_stats using GIN(viewed_but_not_started);
CREATE index if NOT EXISTS user_stats_started_but_not_finished_idx ON user_stats using GIN(started_but_not_finished);
CREATE index if NOT EXISTS user_stats_finished_idx ON user_stats using GIN(finished);
CREATE index if NOT EXISTS user_stats_last_email_notification_idx ON user_stats(last_email_notification);
CREATE index if NOT EXISTS user_stats_last_in_app_notification_idx ON user_stats(last_in_app_notification);
CREATE index if NOT EXISTS user_stats_last_sms_notification_idx ON user_stats(last_sms_notification);
