--- _world

CREATE TABLE _world (
  key VARCHAR(255) NOT NULL PRIMARY KEY,
  value TEXT NOT NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER update_world_updated_at_step1
  BEFORE UPDATE ON world FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_world_updated_at_step2
  BEFORE UPDATE OF updated_at ON world FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_world_updated_at_step3
  BEFORE UPDATE ON world FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_current();

--- users

CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  email VARCHAR(255) NOT NULL,
  encrypted_password_hash VARCHAR(255) NOT NULL,
  encrypted_password_salt VARCHAR(255) NOT NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX index_email_for_users ON users (email);

CREATE TRIGGER update_users_updated_at_step1
  BEFORE UPDATE ON users FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_users_updated_at_step2
  BEFORE UPDATE OF updated_at ON users FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_users_updated_at_step3
  BEFORE UPDATE ON users FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_current();
