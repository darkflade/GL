use std::sync::Arc;
use crate::application::use_cases::files::GetFileUseCase;
use crate::application::use_cases::posts::{CreatePostUseCase, GetAllPostsUseCase, GetPostUseCase, SearchPostsUseCase};
use crate::application::use_cases::tags::SearchTagsUseCase;
use crate::domain::files::FileStorage;
use crate::domain::repository::{FileRepository, PostRepository, TagRepository};
use crate::storage::file_storage::files::LocalFileStorage;
use crate::storage::postgres::files::PostgresFileRepository;
use crate::storage::postgres::posts::PostgresPostRepository;
use crate::storage::postgres::tags::PostgresTagRepository;

pub struct Services<PR, TR, FR, FS> {
    pub create_post: CreatePostUseCase<PR, TR, FR, FS>,
    pub search_posts: SearchPostsUseCase<PR>,
    pub get_post: GetPostUseCase<PR>,
    pub get_all_posts: GetAllPostsUseCase<PR>,
    pub get_file: GetFileUseCase<FR>,
    pub search_tags: SearchTagsUseCase<TR>,
    // ... плейлисты и прочее
}

impl <PR, TR, FR, FS> Services<PR, TR, FR, FS>
where
    PR: PostRepository + Clone + Send + Sync + 'static,
    TR: TagRepository + Clone + Send + Sync + 'static,
    FR: FileRepository + Clone + Send + Sync + 'static,
    FS: FileStorage + Clone + Send + Sync + 'static,
{
    pub fn new(posts: PR, tags: TR, files: FR, storage: FS) -> Self {
        Self {
            create_post: CreatePostUseCase {
                posts: posts.clone(),
                tags: tags.clone(),
                files: files.clone(),
                storage: storage.clone()
            },
            get_post : GetPostUseCase { repo: posts.clone() },
            get_all_posts : GetAllPostsUseCase { repo: posts.clone() },
            search_posts : SearchPostsUseCase { repo: posts.clone() },
            get_file : GetFileUseCase { repo: files },
            search_tags: SearchTagsUseCase { repo: tags },
        }
    }
}

