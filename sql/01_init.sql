create table if not exists projects (
    name text PRIMARY KEY,
    description text,
    created integer default current_timestamp not null
);

create table if not exists log_entries (
    id integer primary key,
    hours real not null,
    description text,
    created integer default current_timestamp not null,
    project text NOT NULL,
    FOREIGN KEY (project) REFERENCES projects(name)
);
