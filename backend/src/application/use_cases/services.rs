use std::sync::Arc;
use crate::application::use_cases::files::GetFileUseCase;
use crate::application::use_cases::playlists::{GetAllPlaylistsUseCase, GetPlaylistUseCase, SearchPlaylistsUseCase};
use crate::application::use_cases::posts::{CreatePostUseCase, GetAllPostsKeysetUseCase, GetAllPostsUseCase, GetPostUseCase, SearchPostsKeysetUseCase, SearchPostsUseCase};
use crate::application::use_cases::tags::SearchTagsUseCase;
use crate::domain::files::FileStorage;
use crate::domain::repository::{FileRepository, PlaylistRepository, PostRepository, TagRepository};

pub struct Services<PR, PLR, TR, FR, FS> {
    //  Posts
    pub create_post: CreatePostUseCase<PR, TR, FR, FS>,
    pub search_posts: SearchPostsUseCase<PR>,
    pub search_posts_keyset: SearchPostsKeysetUseCase<PR>,
    pub get_post: GetPostUseCase<PR>,
    pub get_all_posts: GetAllPostsUseCase<PR>,
    pub get_all_posts_keyset: GetAllPostsKeysetUseCase<PR>,
    //  Playlists
    pub get_playlist: GetPlaylistUseCase<PLR>,
    pub search_playlists: SearchPlaylistsUseCase<PLR>,
    pub get_all_playlists: GetAllPlaylistsUseCase<PLR>,
    //  Tags
    pub search_tags: SearchTagsUseCase<TR>,
    //  Files
    pub get_file: GetFileUseCase<FR>,
}

impl <PR, PLR, TR, FR, FS> Services<PR, PLR, TR, FR, FS>
where
    PR:     PostRepository + Clone + Send + Sync + 'static,
    TR:     TagRepository + Clone + Send + Sync + 'static,
    FR:     FileRepository + Clone + Send + Sync + 'static,
    PLR:    PlaylistRepository + Clone + Send + Sync + 'static,
    FS:     FileStorage + Clone + Send + Sync + 'static,
{
    pub fn new(posts: PR, playlist: PLR, tags: TR, files: FR, storage: FS) -> Self {
        Self {
            //  Posts
            create_post: CreatePostUseCase {
                posts: posts.clone(),
                tags: tags.clone(),
                files: files.clone(),
                storage: storage.clone()
            },
            get_post : GetPostUseCase { repo: posts.clone() },
            get_all_posts : GetAllPostsUseCase { repo: posts.clone() },
            search_posts : SearchPostsUseCase { repo: posts.clone() },
            search_posts_keyset: SearchPostsKeysetUseCase { repo: posts.clone() },
            get_all_posts_keyset: GetAllPostsKeysetUseCase { repo: posts.clone() },
            //  Playlist
            get_playlist: GetPlaylistUseCase { repo: playlist.clone() },
            search_playlists: SearchPlaylistsUseCase { repo: playlist.clone() },
            get_all_playlists: GetAllPlaylistsUseCase { repo: playlist.clone() },
            //  Tags
            search_tags: SearchTagsUseCase { repo: tags },
            //  Files
            get_file : GetFileUseCase { repo: files },
        }
    }
}
