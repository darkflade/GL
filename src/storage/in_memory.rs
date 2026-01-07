use async_trait::async_trait;
use crate::domain::model::{File, Playlist, Post, PostID, PostNote, Tag, TagQuery};
use crate::domain::repository::{PostRepository};
/*
pub struct InMemoryPostRepository {
    posts: Vec<Post>,
}
pub struct InMemoryTagRepository {
    tags: Vec<Tag>,
}
pub struct InMemoryFileRepository {
    handlers: Vec<File>,
}

impl InMemoryPostRepository {
    pub fn new(posts: Vec<Post>) -> Self {
        Self { posts }
    }
}

#[async_trait]
impl PostRepository for InMemoryPostRepository {
    async fn get(&self, id: PostID) -> Result<Post, RepoError> {
        self.posts
            .iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or(RepoError::NotFound)
    }

    async fn search(&self, query: TagQuery) -> Result<Vec<Post>, RepoError> {
        Ok(self.posts
            .iter()
            .filter(|post| {
                query.must.iter().all(|t| post.tag_ids.contains(t))
                    && query.must_not.iter().all(|t| !post.tag_ids.contains(t))
                    && (query.should.is_empty()
                    || query.should.iter().any(|t| post.tag_ids.contains(t)))
            })
            .cloned()
            .collect())
    }
}
*/

