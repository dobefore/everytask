create table if not exists check_expect_task (
id integer PRIMARY KEY UNIQUE,
date_ VARCHAR(30),
task VARCHAR(255),
states tinyint,
descriptions VARCHAR(255)
)