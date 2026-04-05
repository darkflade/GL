# Playlists API Contract

Базовый префикс: `/api/playlists`

Авторизация: cookie-сессия (`actix-identity`), иначе `401`.

Формат ошибок для всех ручек:

```json
{ "error": "..." }
```

## 1) Создать плейлист

`POST /api/playlists`

### Body

```json
{
  "title": "string",
  "description": "string | null",
  "tag_ids": ["uuid"],
  "cover": "uuid | null",
  "items": [
    {
      "position": 1,
      "content": {
        "type": "post",
        "post_id": "uuid"
      }
    },
    {
      "position": 2,
      "content": {
        "type": "note",
        "text": "some note"
      }
    }
  ]
}
```

### Response

- `201 Created`

```json
"playlist_uuid"
```

## 2) Получить плейлист

`GET /api/playlists/{id}`

### Response

- `200 OK`

```json
{
  "id": "uuid",
  "title": "My playlist",
  "description": "...",
  "tags": [
    {
      "id": "uuid",
      "name": "tag",
      "category": "General",
      "count": 123
    }
  ],
  "cover": "uuid",
  "items": [
    {
      "id": "uuid",
      "position": 1,
      "content": {
        "Post": {
          "id": "uuid",
          "title": "post title",
          "description": "...",
          "file": {
            "id": "uuid",
            "path": "rel/path",
            "hash": "...",
            "media_type": "Picture",
            "meta": {
              "width": 1000,
              "height": 800,
              "extension": "jpg",
              "duration_ms": null
            },
            "created_at": "2026-03-03T10:00:00Z",
            "thumbnail": null
          },
          "tags": [],
          "notes": []
        }
      }
    },
    {
      "id": "uuid",
      "position": 2,
      "content": {
        "Note": "text"
      }
    }
  ]
}
```

Важно: наружу отдается числовая `position` (1..N), хотя внутри БД используется строковый `rank`.

## 3) Обновить плейлист

`PATCH /api/playlists/{id}`

Поддерживаются обновления меты плейлиста и событийная обработка элементов.

### Body

```json
{
  "title": "optional",
  "description": "optional | null",
  "cover": "optional_uuid | null",
  "add_tag_ids": ["uuid"],
  "remove_tag_ids": ["uuid"],
  "item_events": [
    {
      "op": "edit",
      "item_id": "uuid",
      "content": {
        "type": "post",
        "post_id": "uuid"
      }
    },
    {
      "op": "move",
      "item_id": "uuid",
      "after_id": "uuid_or_null"
    },
    {
      "op": "add",
      "after_id": "uuid_or_null",
      "content": {
        "type": "note",
        "text": "new note"
      }
    },
    {
      "op": "remove",
      "item_id": "uuid"
    }
  ]
}
```

### Семантика `after_id`

- `after_id = null`: вставить/переместить в начало.
- `after_id = <item_id>`: вставить/переместить сразу после указанного элемента.
- Если `after_id` не найден в плейлисте, сервер вернет `404`.

### Порядок обработки `item_events` на сервере

Сервер выполняет события в 3 фазы:

1. `move` + `edit`
2. `add`
3. `remove`

Это гарантирует корректный reordering + batch insert/delete за один запрос.

### Response

- `204 No Content`

## 4) Удалить плейлист

`DELETE /api/playlists/{id}`

### Response

- `204 No Content`

## 5) Поиск моих плейлистов

`GET /api/playlists/search`

Текущая реализация читает JSON body даже для `GET`.

### Body

```json
{
  "text_query": "optional text",
  "tag_query": {
    "must": ["tag"],
    "should": ["tag"],
    "must_not": ["tag"]
  },
  "cursor": {
    "mode": "keyset",
    "last_id": "uuid",
    "last_score": 10.5,
    "limit": 30,
    "direction": "next"
  }
}
```

### Response

- `200 OK`

```json
{
  "playlists": [
    {
      "id": "uuid",
      "title": "title",
      "description": "desc",
      "cover": "uuid",
      "item_count": 10,
      "tags": []
    }
  ],
  "has_next": true,
  "has_prev": false,
  "next_cursor": {
    "mode": "keyset",
    "direction": "next",
    "last_id": "uuid",
    "last_score": 1.23,
    "limit": 30
  },
  "prev_cursor": null
}
```

## Подсказка для фронта

Для drag-and-drop удобно хранить у себя список `item_id` в текущем порядке и отправлять `move/add` только через `after_id`, без попыток вычислять внутренний `rank`.
