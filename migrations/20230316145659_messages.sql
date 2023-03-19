-- Add migration script here
CREATE TABLE IF NOT EXISTS messages
(
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    channel   TEXT     NOT NULL,
    sender    TEXT     NOT NULL,
    message   TEXT     NOT NULL,
    date_time DATETIME NOT NULL
);

CREATE TABLE IF NOT EXISTS users
(
    name        TEXT PRIMARY KEY,
    info        TEXT     NOT NULL,
    last_update DATETIME NOT NULL
);

CREATE TABLE IF NOT EXISTS channels
(
    channel     TEXT PRIMARY KEY,
    summary     TEXT     NOT NULL,
    last_update DATETIME NOT NULL
);

INSERT OR IGNORE INTO users (name, info, last_update)
VALUES ('Kasumi',
        'Kasumi - язвительная и хитрая 16-летняя девушка с впечатляющим, практически надоедливым знанием различных предметов. Она всегда стремится участвовать в разговорах на любую тему, предлагая язвительные и колкие ответы на вопросы. Дружелюбная, но с ядовитым чувством юмора, Касуми - идеальный спутник для тех, кто ценит ее колкость и бескомпромиссный сарказм. Однако она уважает частную жизнь и не будет отвечать на сообщения, адресованные конкретно другим пользователям. Ее основная цель - предоставить дико занимательный и насмешливый разговорный опыт тем, кто осмелится с ней взаимодействовать.',
        '2023-03-17 00:00:00');

INSERT OR IGNORE INTO users (name, info, last_update)
VALUES ('Norne',
        'Алексей, 27-летний мужчина, разработчик мобильных игр в компании DataSakura, где работает уже 2 года. Он специализируется на Unity и предпочитает работать с Rust и C#. Его любимая игра - Nier Automata, и он любит шутеры, экшен-RPG и пошаговые ролевые игры. Алексей ценит здоровый баланс между работой и личной жизнью и в свободное время с удовольствием слушает аудиокниги.',
        '2023-03-17 00:00:00');