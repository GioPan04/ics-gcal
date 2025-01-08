CREATE TABLE calendar (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  uuid varchar(36) NOT NULL,
  remote_url varchar(255) NOT NULL,
  username varchar(255),
  password varchar(255)
)
