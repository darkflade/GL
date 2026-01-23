# Endpoints:

## Playlists
___
### [Get]
- playlists (in cookie user determined)
- playlists/{id}
### [Post]
- playlists
- playlists/{id}/item
### [Patch]
- playlists/{id}
### [Delete]
- playlist/{id}

## Posts
___
### [Get]
- posts (tags in query)
- posts/{id}

### [Post]
- posts

### [Delete]
- posts/{id}



    GET /playlists — Список плейлистов юзера (Краткие карточки PlaylistSummary).
    GET /playlists/{id} — Полный плейлист (PlaylistWithItems).
    POST /playlists — Создать новый плейлист.
    PATCH /playlists/{id} — Обновить метаданные (название, обложка).
    DELETE /playlists/{id} — Удалить плейлист.
    POST /playlists/{id}/items — Добавить элемент в плейлист.

    GET /posts — Поиск постов (с Query Params: ?tags=...&page=1).
    GET /posts/{id} — Получить пост (метаданные).
    POST /posts — Создать пост (Загрузка файла + JSON).
    DELETE /posts/{id} — Удалить.
  
    GET /files/{id}