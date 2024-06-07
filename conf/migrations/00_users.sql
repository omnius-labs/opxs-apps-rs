-- users

CREATE TYPE user_authentication_type AS ENUM ('Email', 'Provider');
CREATE TYPE user_role AS ENUM ('Admin', 'User');

CREATE TABLE users (
    id VARCHAR(255) NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    authentication_type user_authentication_type NOT NULL,
    role user_role NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
);
