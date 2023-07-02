--- users

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    authentication_type VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    password_hash VARCHAR(255) NOT NULL,
    salt VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER update_users_updated_at_step1
  BEFORE UPDATE ON users FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_users_updated_at_step2
  BEFORE UPDATE OF updated_at ON users FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_users_updated_at_step3
  BEFORE UPDATE ON users FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_current();

--- refresh_tokens

CREATE TABLE refresh_tokens (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);

CREATE TRIGGER update_refresh_tokens_updated_at_step1
  BEFORE UPDATE ON refresh_tokens FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_refresh_tokens_updated_at_step2
  BEFORE UPDATE OF updated_at ON refresh_tokens FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_refresh_tokens_updated_at_step3
  BEFORE UPDATE ON refresh_tokens FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_current();
