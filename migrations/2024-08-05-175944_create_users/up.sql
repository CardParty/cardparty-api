-- Your SQL goes here
create table users
(
    user_id  uuid    not null
        constraint users_pk
            primary key,
    username varchar not null
);