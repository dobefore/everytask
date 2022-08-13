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
);
create table if not exists everydaytask (
id integer PRIMARY KEY UNIQUE,
date_ VARCHAR(30),
getup_ts VARCHAR(30),
bed_ts VARCHAR(30),
day_dur integer ,
begin_ts VARCHAR(30),
end_ts VARCHAR(30),
one_task_dur integer,
task VARCHAR(255),
detail VARCHAR(255),
ex_task VARCHAR(255),
ex_detail VARCHAR(300)
);

create table if not exists pay (
    date_ VARCHAR(30),
    item VARCHAR(30),
    -- be string in case infinite fraction
    price VARCHAR(30),
    amounts integer ,
    category VARCHAR(30)
)