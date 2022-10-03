 create table if not exists test (
  name varchar(255) NOT NULL,
  queries text NOT NULL,
  executed_at timestamp without time zone default CURRENT_TIMESTAMP,
  primary key (name)
);
