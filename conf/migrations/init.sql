 CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  email VARCHAR(255) NOT NULL,
  password VARCHAR(255) NOT NULL,
  salt VARCHAR(255) NOT NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE tweets (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL,
  text VARCHAR(4096) NOT NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE follows (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL,
  follow_user_id BIGINT NOT NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE likes (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL,
  tweet_id BIGINT NOT NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE sessions (
  id bytea PRIMARY KEY,
  user_id BIGINT NOT NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- FUNCTION

CREATE FUNCTION refresh_updated_at_none() RETURNS trigger AS
$$
BEGIN
  IF NEW.updated_at = OLD.updated_at THEN
    NEW.updated_at := NULL;
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION refresh_updated_at_same() RETURNS trigger AS
$$
BEGIN
  IF NEW.updated_at IS NULL THEN
    NEW.updated_at := OLD.updated_at;
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION refresh_updated_at_current() RETURNS trigger AS
$$
BEGIN
  IF NEW.updated_at IS NULL THEN
    NEW.updated_at := CURRENT_TIMESTAMP;
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- TRIGGER

CREATE TRIGGER update_users_updated_at_step1
  BEFORE UPDATE ON users FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_users_updated_at_step2
  BEFORE UPDATE OF updated_at ON users FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_users_updated_at_step3
  BEFORE UPDATE ON users FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_current();

CREATE TRIGGER update_tweets_updated_at_step1
  BEFORE UPDATE ON tweets FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_tweets_updated_at_step2
  BEFORE UPDATE OF updated_at ON tweets FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_tweets_updated_at_step3
  BEFORE UPDATE ON tweets FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_current();

CREATE TRIGGER update_follows_updated_at_step1
  BEFORE UPDATE ON follows FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_follows_updated_at_step2
  BEFORE UPDATE OF updated_at ON follows FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_follows_updated_at_step3
  BEFORE UPDATE ON follows FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_current();

CREATE TRIGGER update_likes_updated_at_step1
  BEFORE UPDATE ON likes FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_likes_updated_at_step2
  BEFORE UPDATE OF updated_at ON likes FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_likes_updated_at_step3
  BEFORE UPDATE ON likes FOR EACH ROW
  EXECUTE PROCEDURE refresh_updated_at_current();
