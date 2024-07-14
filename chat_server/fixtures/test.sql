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
(1, 'single', '{2, 4}'),
(1, 'group', '{1, 2, 3}');


-- insert 10 messages
INSERT INTO messages (chat_id, sender_id, content) VALUES
(1, 1, 'Hello Bob!'),
(1, 2, 'Hello Alice!'),
(1, 1, 'How are you?'),
(1, 2, 'I am fine, thank you!'),
(1, 1, 'Good to hear that!'),
(1, 2, 'How about you?'),
(1, 1, 'I am fine too!'),
(1, 2, 'Great!'),
(1, 1, 'Bye!'),
(1, 2, 'Bye!');