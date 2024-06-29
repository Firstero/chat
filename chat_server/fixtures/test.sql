-- insert workspaces
INSERT INTO workspaces (name, owner_id) VALUES ('acme', 0), ('foo', 0), ('bar', 0);

-- insert 5 users all them with the same password 123456
INSERT INTO users (ws_id, fullname, email, password_hash) VALUES
(1, 'Alice', 'Alice@test.org', '$argon2id$v=19$m=19456,t=2,p=1$CsXTc5LX86DPIp0OFbLg/w$aZhy+a3yHCqmd39zYQnY+V/WZX+T5UcHJadv4A8v2/0'),
(1, 'Bob', 'Bob@test.org', '$argon2id$v=19$m=19456,t=2,p=1$CsXTc5LX86DPIp0OFbLg/w$aZhy+a3yHCqmd39zYQnY+V/WZX+T5UcHJadv4A8v2/0'),
(1, 'Charlie', 'Chalie@tes.org', '$argon2id$v=19$m=19456,t=2,p=1$CsXTc5LX86DPIp0OFbLg/w$aZhy+a3yHCqmd39zYQnY+V/WZX+T5UcHJadv4A8v2/0'),
(1, 'David', 'David@test.org', '$argon2id$v=19$m=19456,t=2,p=1$CsXTc5LX86DPIp0OFbLg/w$aZhy+a3yHCqmd39zYQnY+V/WZX+T5UcHJadv4A8v2/0'),
(1, 'Eve', 'Eve@test.org', '$argon2id$v=19$m=19456,t=2,p=1$CsXTc5LX86DPIp0OFbLg/w$aZhy+a3yHCqmd39zYQnY+V/WZX+T5UcHJadv4A8v2/0');

-- insert chats
INSERT INTO chats (ws_id, name, type, members) VALUES
(1, 'Private Channel', 'private_channel', '{1, 2}'),
(1, 'Public Channel', 'public_channel', '{1, 2, 3, 4, 5}');

-- insert unnamed chat
INSERT INTO chats (ws_id, type, members) VALUES
(1, 'single', '{1, 2}'),
(1, 'single', '{1, 3}'),
(1, 'single', '{1, 4}'),
(1, 'single', '{1, 5}'),
(1, 'group', '{1, 2, 3}'),
(1, 'group', '{1, 2, 3, 4}'),
(1, 'group', '{1, 2, 3, 4, 5}');