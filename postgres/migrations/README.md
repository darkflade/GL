Для существующей БД остановите backend и примените миграцию до запуска новой версии:

```sh
docker compose stop backend
docker compose exec -T postgres psql -U glab -d glab -v ON_ERROR_STOP=1 < postgres/migrations/20261116_collections.sql
docker compose up -d --build backend frontend
```

Миграция выполняется один раз, в транзакции. Она сохраняет данные, идентификаторы и порядок элементов. Для новой БД используется `postgres/schema.sql`, миграцию поверх него выполнять не нужно. Сделайте резервную копию существующей БД перед обновлением.

API и страницы переименованы: `/playlists` → `/collections`, `/playlist` → `/collection`; поле ответа `playlists` → `collections`. Клиенты нужно обновить вместе с backend.

Для настроек воспроизведения после миграции коллекций примените:

```sh
docker compose exec -T postgres psql -U glab -d glab -v ON_ERROR_STOP=1 < postgres/migrations/20261116_playback.sql
```

Новая БД уже содержит таблицу `playback_settings` в `schema.sql`.

Для сохранения позиции презентации после миграции playback:

```sh
docker compose exec -T postgres psql -U glab -d glab -v ON_ERROR_STOP=1 < postgres/migrations/20261116_playback_position.sql
```
