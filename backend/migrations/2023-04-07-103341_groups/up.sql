CREATE TABLE groups (
  uuid TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NUll UNIQUE,
  faculty TEXT NOT NULL,
  FOREIGN KEY(faculty) REFERENCES faculties (uuid)
);
