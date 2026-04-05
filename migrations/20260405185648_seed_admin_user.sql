INSERT INTO users (username, password_hash, display_name)
VALUES ('admin', '$argon2id$v=19$m=19456,t=2,p=1$Bx+USQ61TTcie10RqCxU4Q$midixmMPzJWresAqWJHiHE9oToxjpQS0OGgHxht8M/U', 'Administrator');

INSERT INTO user_groups (user_id, group_id)
SELECT u.id, g.id
FROM users u, groups g
WHERE u.username = 'admin' AND g.name = 'admin';