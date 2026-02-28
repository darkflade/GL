use crate::application::ports::{
    FileRepository, PlaylistRepository, PostRepository, TagRepository,
};
use crate::application::use_cases::files::GetFileUseCase;
use crate::application::use_cases::playlists::{
    CreatePlaylistUseCase, DeletePlaylistUseCase, GetAllPlaylistsUseCase, GetPlaylistUseCase,
    SearchPlaylistsUseCase, UpdatePlaylistUseCase,
};
use crate::application::use_cases::posts::{
    CreatePostUseCase, DeletePostUseCase, GetAllPostsKeysetUseCase, GetAllPostsUseCase,
    GetPostUseCase, SearchPostsKeysetUseCase, SearchPostsUseCase, UpdatePostUseCase,
};
use crate::application::use_cases::tags::SearchTagsUseCase;
use crate::domain::files::FileStorage;
pub struct Services<PR, PLR, TR, FR, FS> {
    //  Posts
    pub create_post: CreatePostUseCase<PR, TR, FR, FS>,
    pub search_posts: SearchPostsUseCase<PR>,
    pub search_posts_keyset: SearchPostsKeysetUseCase<PR>,
    pub get_post: GetPostUseCase<PR>,
    pub delete_post: DeletePostUseCase<PR>,
    pub update_post: UpdatePostUseCase<PR>,
    pub get_all_posts: GetAllPostsUseCase<PR>,
    pub get_all_posts_keyset: GetAllPostsKeysetUseCase<PR>,
    //  Playlists
    pub get_playlist: GetPlaylistUseCase<PLR>,
    pub create_playlist: CreatePlaylistUseCase<PLR>,
    pub delete_playlist: DeletePlaylistUseCase<PLR>,
    pub update_playlist: UpdatePlaylistUseCase<PLR>,
    pub search_playlists: SearchPlaylistsUseCase<PLR>,
    pub get_all_playlists: GetAllPlaylistsUseCase<PLR>,
    //  Tags
    pub search_tags: SearchTagsUseCase<TR>,
    //  Files
    pub get_file: GetFileUseCase<FR>,
}

impl<PR, PLR, TR, FR, FS> Services<PR, PLR, TR, FR, FS>
where
    PR: PostRepository + Clone + Send + Sync + 'static,
    TR: TagRepository + Clone + Send + Sync + 'static,
    FR: FileRepository + Clone + Send + Sync + 'static,
    PLR: PlaylistRepository + Clone + Send + Sync + 'static,
    FS: FileStorage + Clone + Send + Sync + 'static,
{
    pub fn new(posts: PR, playlist: PLR, tags: TR, files: FR, storage: FS) -> Self {
        Self {
            //  Posts
            create_post: CreatePostUseCase {
                posts: posts.clone(),
                tags: tags.clone(),
                files: files.clone(),
                storage: storage.clone(),
            },
            get_post: GetPostUseCase {
                repo: posts.clone(),
            },
            delete_post: DeletePostUseCase {
                repo: posts.clone(),
            },
            update_post: UpdatePostUseCase {
                repo: posts.clone(),
            },
            get_all_posts: GetAllPostsUseCase {
                repo: posts.clone(),
            },
            search_posts: SearchPostsUseCase {
                repo: posts.clone(),
            },
            search_posts_keyset: SearchPostsKeysetUseCase {
                repo: posts.clone(),
            },
            get_all_posts_keyset: GetAllPostsKeysetUseCase {
                repo: posts.clone(),
            },
            //  Playlist
            get_playlist: GetPlaylistUseCase {
                repo: playlist.clone(),
            },
            create_playlist: CreatePlaylistUseCase {
                repo: playlist.clone(),
            },
            delete_playlist: DeletePlaylistUseCase {
                repo: playlist.clone(),
            },
            update_playlist: UpdatePlaylistUseCase {
                repo: playlist.clone(),
            },
            search_playlists: SearchPlaylistsUseCase {
                repo: playlist.clone(),
            },
            get_all_playlists: GetAllPlaylistsUseCase {
                repo: playlist.clone(),
            },
            //  Tags
            search_tags: SearchTagsUseCase { repo: tags },
            //  Files
            get_file: GetFileUseCase { repo: files },
        }
    }
}
