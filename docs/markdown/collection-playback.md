# Collection playback

Click a collection item or Present collection to open the viewer. Arrow keys navigate; Space toggles slideshow; Escape closes settings first, then the viewer. Fullscreen hides browser chrome where supported. Images fit the available screen without cropping. Navigation uses the configured media skips; a clicked item can always be opened explicitly.

Settings are personal to the authenticated user. A collection override replaces the full global configuration; missing global settings use server defaults. Use global settings deletes the collection override. Changing global settings does not overwrite existing collection overrides.

Defaults: 5-second interval, forward, looping, audio slides skipped, videos included, video muted while background audio plays, next track/slide after media completion. Images, notes and included audio slides use the timer. Videos wait until they finish. Background audio continues independently of slideshow pause and image navigation; audio controls can pause it. Closing the viewer stops all media. Browser autoplay restrictions may require starting media through its controls.

`GET /api/playback?collection_id=<uuid>` returns `{global, collection}` where collection is nullable. `PUT /api/playback` saves global settings; `PUT /api/playback?collection_id=<uuid>` saves the collection override. `DELETE` with collection_id resets the override; without it resets globals. Collection access requires ownership; global settings only affect the current user.

Settings JSON fields: `interval_seconds` (integer 1–3600), `direction` (`forward`/`backward`), `loop_collection`, `skip_audio`, `skip_video`, `mute_video_with_audio` (booleans), `audio_end` and `video_end` (`repeat`/`next`/`stop`). Audio next follows the collection's audio tracks in the configured direction. Global/collection looping also controls the audio queue.

Apply `postgres/migrations/20261116_playback.sql` once to an existing database after the collection rename migration, before starting the updated backend. Fresh installations use `postgres/schema.sql`.

The Item number / Go control jumps directly to an element, including skipped audio/video. The current element is saved per user and collection after navigation and on close. Present collection resumes that item; clicking an individual card opens the clicked item. Positions use collection-item UUIDs, so reorder preserves the bookmark, while deleting the bookmarked item clears it.

`GET /api/playback?collection_id=<uuid>` also returns nullable `item_id`. `PUT /api/playback/position?collection_id=<uuid>` accepts `{ "item_id": "<collection-item-uuid>" }`; it requires collection ownership and verifies that the item belongs to that collection. Apply `20261116_playback_position.sql` to existing databases.

`shortcut_item_id` is an optional collection playback setting. Choose Quick-jump button target in this collection's playback settings to show a separate floating button at the lower right. It jumps to that saved element and pauses slideshow so skipped media can also be inspected. The shortcut is independent of both current/resume position and the Item/Go control. Global settings cannot contain a collection-item shortcut. Deleting its target hides the shortcut; a new target can be selected in settings. This setting uses the existing JSON storage, so no new migration is needed.
