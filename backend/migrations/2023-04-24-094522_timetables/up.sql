CREATE TABLE timetables (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  name TEXT NOT NULL,
  day TEXT NOT NULL,
  start_time TEXT NOT NULL,
  end_time TEXT NOT NULL,
  student_group TEXT NOT NULL,
  FOREIGN KEY(student_group) REFERENCES groups (uuid)
);
