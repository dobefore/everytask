create table if not exists everytask (
id integer PRIMARY KEY UNIQUE,
date_ VARCHAR(30),
getup_ts VARCHAR(30),
bed_ts VARCHAR(30),
day_dur integer ,
begin_ts VARCHAR(30),
end_ts VARCHAR(30),
one_task_dur integer,
task VARCHAR(255),
detail VARCHAR(255)
)