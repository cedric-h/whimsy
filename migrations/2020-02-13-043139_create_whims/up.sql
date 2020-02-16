CREATE TABLE whims (
  title TEXT NOT NULL PRIMARY KEY,
  body TEXT NOT NULL
);

INSERT INTO whims (title, body) VALUES ("hi", "rect(0, 0, 100, 100)");
INSERT INTO whims (title, body) VALUES ("sweep", "rect(wave(100), 0, 100, 100)");
