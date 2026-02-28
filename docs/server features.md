# First
[ ] Logic for delete and edit 
  posts/playlists/tags
  Endpoints: DELETE /posts/:id, PATCH /posts/:id, POST /tags, PATCH /tags/:id, DELETE /tags/:id.
  Post deleting: transaction post_tags, tags.post_count--, create task for deleting, delete from playlist.

[ ] Playlists system
  Index: (playlist_id, position)
  For reorder: get some space (start 1024). Ends — реassign positions.
  Endpoints: create playlist, add item (at position), remove item, reorder (PATCH reorder with array of ids or move item API).

[ ] Thumbnails system
  Save the original and then create Task(Reddis, RocketMQ, RabbitMQ)
  thumb (libvip/ImageMagick; video — ffmpeg snapshot) → atomic fs::rename.

  Paths: 
    - /files/<uuid>.ext, 
    - /thumbs/<uuid>.webp
  Logic as files for get
### Prob
[x] Cursor return previous

## Secodary
[ ] Logger(made some improvments) 

[ ] do something with unused tags 
  Periodicaly taks (cron/background job) for deleting tags WHERE post_count = 0 AND created_at < now() - interval '30 days' (or delete).
  Load testing: k6, wrk/hey — imitate reads/writes.
  
[ ] Health checks

### Optionaly  
[ ] Workers for magic bytes/phash/md5
[ ] Metrics & monitoring
  Backend: Prometheus + Grafana (expose /metrics via prometheus crate).
  Error tracking: Sentry.
[ ] Copy tags posts\playlists
[ ] Group by similarity (thumbs speed up mb)
[ ] connected tags\synonims
  UI: hint "relative tags/ similar" + checkbox "enable parent".
[ ] statistic
[ ] grpc
[ ] assemble docs




#### Profiling: tokio-console, pprof, perf.
#### Atomic counters / materialized views
#### post_count\materialized view\periodically update.
#### Watch recomendations
#### RAG



