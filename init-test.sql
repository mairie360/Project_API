-- Test fixtures run by the `seeder` service once Liquibase is done (dev, integration,
-- performance and security stacks).
--
-- User 1 (Admin) is already created by the `create_admin` changeset of the liquibase-migrations
-- image; it is the `sub` of the JWT ZAP injects. Users 2 and 3 are plain `User` accounts (project
-- members, collaborators).
--
-- User 42, project 12 and task 87 are the ids of the path parameter examples of the spec: ZAP
-- builds its requests from these examples, so seeding them makes it scan the handlers on real
-- rows instead of stopping at a 404.
--
-- The password is the public argon2id hash of the template admin account: `users.password` only
-- accepts argon2id PHC strings since database 1.3.0 (chk_users_password_hashed).

INSERT INTO users (id, first_name, last_name, email, password, status, is_archived)
VALUES
    (2, 'Test', 'User', 'test2@mairie360.fr',
     '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw',
     'active', FALSE),
    (3, 'Test', 'User', 'test3@mairie360.fr',
     '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw',
     'active', FALSE),
    (42, 'Jean', 'Dupont', 'jean.dupont@mairie360.fr',
     '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw',
     'active', FALSE)
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id FROM roles r CROSS JOIN (VALUES (2), (3), (42)) AS u(id) WHERE r.name = 'User'
ON CONFLICT DO NOTHING;

INSERT INTO projects (id, title, description, owner_id)
VALUES (12, 'Réfection de la place du marché', 'Travaux de voirie 2026', 1)
ON CONFLICT (id) DO NOTHING;

INSERT INTO project_members (project_id, user_id) VALUES (12, 42)
ON CONFLICT DO NOTHING;

INSERT INTO tasks (id, project_id, title, status, priority, due_date, assigned_to)
VALUES (87, 12, 'Consulter les riverains', 'in_progress', 'high', '2026-10-15', 42)
ON CONFLICT (id) DO NOTHING;

-- Explicit ids do not advance the SERIAL sequence: move it past the fixtures so users created
-- later (Core_API, other fixtures) do not collide with them.
SELECT setval(pg_get_serial_sequence('users', 'id'), GREATEST((SELECT MAX(id) FROM users), 1));
SELECT setval(pg_get_serial_sequence('projects', 'id'), GREATEST((SELECT MAX(id) FROM projects), 1));
SELECT setval(pg_get_serial_sequence('tasks', 'id'), GREATEST((SELECT MAX(id) FROM tasks), 1));
