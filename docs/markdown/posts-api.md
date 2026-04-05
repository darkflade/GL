# Posts API Contract

Базовый префикс: `/api/posts`

Авторизация для постов сейчас не требуется на уровне хендлеров.

Формат ошибок:

```json
{ "error": "..." }
```

## 1) Создать пост

`POST /api/posts`

Тип запроса: `multipart/form-data`

Поля multipart:
- `meta` (JSON-строка)
- `file` (бинарный файл)

Важно: `meta` должен идти раньше `file`, иначе сервер вернет `400`.

### `meta` формат

```json
{
  "title": "My post",
  "tags": ["landscape", "night"]
}
```

### Response

- `201 Created`

```json
"post_uuid"
```

## 2) Получить пост

`GET /api/posts/{id}`

### Response

- `200 OK`

```json
{
  "id": "uuid",
  "title": "Post title",
  "description": "optional",
  "file": {
    "id": "uuid",
    "path": "rel/path/file.jpg",
    "hash": "optional",
    "media_type": "Picture",
    "meta": {
      "width": 1920,
      "height": 1080,
      "extension": "jpg",
      "duration_ms": null
    },
    "created_at": "2026-03-06T10:00:00Z",
    "thumbnail": null
  },
  "tags": [
    {
      "id": "uuid",
      "name": "landscape",
      "category": "General",
      "count": 10
    }
  ],
  "notes": [
    {
      "id": "uuid",
      "text": "note",
      "x": 0.25,
      "y": 0.75
    }
  ]
}
```

## 3) Обновить пост

`PATCH /api/posts/{id}`

### Body

```json
{
  "title": "optional",
  "description": "optional | null",
  "add_tag_ids": ["uuid"],
  "remove_tag_ids": ["uuid"],
  "notes": [
    {
      "id": "optional_uuid",
      "text": "text",
      "x": 0.1,
      "y": 0.2
    }
  ]
}
```

Поведение по `notes`: сервер сейчас удаляет все старые заметки поста и вставляет новые из массива.

### Response

- `204 No Content`

## 4) Удалить пост

`DELETE /api/posts/{id}`

### Response

- `204 No Content`

## 5) Поиск постов

`POST /api/posts/search`

### Body

```json
{
  "text_query": "optional",
  "tag_query": {
    "must": ["tag1"],
    "should": ["tag2"],
    "must_not": ["tag3"]
  },
  "cursor": {
    "mode": "keyset",
    "page": 0,
    "last_id": "uuid",
    "last_score": 1.0,
    "limit": 30,
    "direction": "next"
  }
}
```

Важно:
- Для `mode = offset` используется `page`.
- Для `mode = keyset` используются `last_id/last_score/limit/direction`.
- `text_query` в текущей серверной реализации постов не участвует в SQL-фильтрации.

### Response (`offset`)

```json
{
  "posts": [
    {
      "id": "uuid",
      "title": "...",
      "description": "...",
      "file": { "id": "uuid", "path": "...", "hash": null, "media_type": "Picture", "meta": null, "created_at": null, "thumbnail": null },
      "tags": [],
      "notes": []
    }
  ],
  "total_pages": 12
}
```

### Response (`keyset`)

```json
{
  "posts": [
    {
      "id": "uuid",
      "title": "...",
      "description": "...",
      "file": { "id": "uuid", "path": "...", "hash": null, "media_type": "Picture", "meta": null, "created_at": null, "thumbnail": null },
      "tags": [],
      "notes": []
    }
  ],
  "has_next": true,
  "has_prev": false,
  "next_cursor": {
    "mode": "keyset",
    "direction": "next",
    "last_id": "uuid",
    "last_score": 2.0,
    "limit": 30
  },
  "prev_cursor": null
}
```
