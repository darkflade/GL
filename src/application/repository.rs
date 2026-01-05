use uuid::Uuid;
use crate::domain::files::FileStorage;
use crate::domain::model::{File, FileID, NewPost, NewTag, Playlist, PlaylistID, PlaylistQuery, Post, PostID, RepoError, Tag, TagQuery};
use crate::domain::repository::{FileRepository, PlaylistRepository, PostRepository, TagRepository};

// Post Use-Case
pub struct CreatePostUseCase<
    PR: PostRepository,
    FS: FileStorage,
    TR: TagRepository,
> {
    posts: PR,
    files: FS,
    tags: TR,
}

impl<PR: PostRepository, FS: FileStorage, TR: TagRepository> CreatePostUseCase<PR, FS, TR> {
    pub async fn execute(
        &self,
        title: String,
        file_bytes: Vec<u8>,
        file_ext: Option<&str>,
        tags: Vec<NewTag>
    ) -> Result<PostID, RepoError> {
        let file_id = self.files.save(&file_bytes, file_ext)
            .await
            .map_err(|_| RepoError::StorageError)?;


        let created_tags = self.tags.get_or_create(tags).await?;
        let tag_ids: Vec<Uuid> = created_tags.into_iter().map(|t| t.id).collect();


        let new_post = NewPost {
            id: Uuid::new_v4(),
            title,
            file_id,
        };

        self.posts.create(new_post, &tag_ids).await
    }
}

pub struct SearchPostsUseCase<R: PostRepository> {
    pub repo: R,
}

pub struct GetPostUseCase<R: PostRepository> {
    pub repo: R,
}

impl<R: PostRepository> SearchPostsUseCase<R> {
    pub async fn execute(&self, query: TagQuery) -> Result<Vec<Post>, RepoError> {
        self.repo.search(query).await
    }
}

impl<R: PostRepository> GetPostUseCase<R> {
    pub async fn execute(&self, id: PostID) -> Result<Post, RepoError> {
        self.repo.get(id).await
    }
}

// Playlist Use-Case
pub struct SearchPlaylistsUseCase<R: PlaylistRepository> {
    pub repo: R,
}

pub struct GetPlaylistUseCase<R: PlaylistRepository> {
    pub repo: R,
}
impl<R: PlaylistRepository> SearchPlaylistsUseCase<R> {

    pub async fn execute(&self, query: PlaylistQuery) -> Result<Vec<Playlist>, RepoError> {
        self.repo.search(query).await
    }
}
impl<R: PlaylistRepository> GetPlaylistUseCase<R> {
    pub async fn execute(&self, id: PlaylistID) -> Result<Playlist, RepoError> {
        self.repo.get(id).await
    }
}

// File Use-Case
pub struct GetFileUseCase<R: FileRepository> {
    pub repo: R,
}

impl<R: FileRepository> GetFileUseCase<R> {
    pub async fn execute(&self, id: FileID) -> Result<File, RepoError> {
        self.repo.get(id).await
    }
}

// Tag Use-Case

pub struct CreateTagUseCase<R: TagRepository> {
    pub repo: R,
}

impl<R: TagRepository> CreateTagUseCase<R> {
    pub async fn execute(&self, tags: Vec<NewTag>) -> Result<Vec<Tag>, RepoError> {
        self.repo.get_or_create(tags).await
    }
}


