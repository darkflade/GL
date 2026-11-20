# Endpoints

Базовый префикс: `/api`

## Posts

### `POST /posts`
- Создание поста через `multipart/form-data`
- Поля: `meta` (JSON-строка), `file` (бинарник)
- `meta` должен идти раньше `file`

### `POST /posts/search`
- Поиск постов
- Body: `text_query`, `tag_query`, `cursor`

### `GET /posts/{id}`
- Получить полный пост

### `PATCH /posts/{id}`
- Обновить метаданные поста, теги и заметки

### `DELETE /posts/{id}`
- Удалить пост

## Tags

### `GET /tags`
- Список тегов с keyset-пагинацией
- Query: `mode`, `last_id`, `last_score`, `limit`, `direction`

### `GET /tags/search`
- Поиск тегов по строке
- Query: `query`

### `PATCH /tags`
- Батч-обновление тегов

### `GET /tags/relations`
- Список связей тегов с keyset-пагинацией
- Query: `mode`, `last_id`, `last_score`, `limit`, `direction`

### `PATCH /tags/relations`
- Батч-обновление связей тегов

### `GET /tags/{id}/related`
- Получить связанные теги для конкретного тега

## Collections

### `POST /collections`
- Создать коллекция

### `POST /collections/search`
- Получить коллекции пользователя

### `GET /collections/{id}`
- Получить полный коллекция

### `PATCH /collections/{id}`
- Обновить коллекция

### `DELETE /collections/{id}`
- Удалить коллекция

## Files

### `GET /files/{id}`
- Скачать файл
