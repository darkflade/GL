use crate::application::ports::{
    FileRepository, CollectionRepository, PostRepository, TagRepository,
};
use crate::application::use_cases::files::GetFileUseCase;
use crate::application::use_cases::collections::{
    CreateCollectionUseCase, DeleteCollectionUseCase, GetAllCollectionsUseCase, GetCollectionUseCase,
    SearchCollectionsUseCase, UpdateCollectionUseCase,
};
use crate::application::use_cases::posts::{
    CreatePostUseCase, DeletePostUseCase, GetAllPostsKeysetUseCase, GetAllPostsUseCase,
    GetPostUseCase, SearchPostsKeysetUseCase, SearchPostsUseCase, UpdatePostUseCase,
};
use crate::application::use_cases::tags::{
    GetRelatedTagsUseCase, ListTagRelationsKeysetUseCase, ListTagsKeysetUseCase, SearchTagsUseCase,
    UpdateTagRelationsUseCase, UpdateTagsUseCase,
};
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
    //  Collections
    pub get_collection: GetCollectionUseCase<PLR>,
    pub create_collection: CreateCollectionUseCase<PLR>,
    pub delete_collection: DeleteCollectionUseCase<PLR>,
    pub update_collection: UpdateCollectionUseCase<PLR>,
    pub search_collections: SearchCollectionsUseCase<PLR>,
    pub get_all_collections: GetAllCollectionsUseCase<PLR>,
    //  Tags
    pub search_tags: SearchTagsUseCase<TR>,
    pub list_tags_keyset: ListTagsKeysetUseCase<TR>,
    pub get_related_tags: GetRelatedTagsUseCase<TR>,
    pub list_tag_relations_keyset: ListTagRelationsKeysetUseCase<TR>,
    pub update_tags: UpdateTagsUseCase<TR>,
    pub update_tag_relations: UpdateTagRelationsUseCase<TR>,
    //  Files
    pub get_file: GetFileUseCase<FR>,
}

impl<PR, PLR, TR, FR, FS> Services<PR, PLR, TR, FR, FS>
where
    PR: PostRepository + Clone + Send + Sync + 'static,
    TR: TagRepository + Clone + Send + Sync + 'static,
    FR: FileRepository + Clone + Send + Sync + 'static,
    PLR: CollectionRepository + Clone + Send + Sync + 'static,
    FS: FileStorage + Clone + Send + Sync + 'static,
{
    pub fn new(posts: PR, collection: PLR, tags: TR, files: FR, storage: FS) -> Self {
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
            //  Collection
            get_collection: GetCollectionUseCase {
                repo: collection.clone(),
            },
            create_collection: CreateCollectionUseCase {
                repo: collection.clone(),
            },
            delete_collection: DeleteCollectionUseCase {
                repo: collection.clone(),
            },
            update_collection: UpdateCollectionUseCase {
                repo: collection.clone(),
            },
            search_collections: SearchCollectionsUseCase {
                repo: collection.clone(),
            },
            get_all_collections: GetAllCollectionsUseCase {
                repo: collection.clone(),
            },
            //  Tags
            search_tags: SearchTagsUseCase { repo: tags.clone() },
            list_tags_keyset: ListTagsKeysetUseCase { repo: tags.clone() },
            get_related_tags: GetRelatedTagsUseCase { repo: tags.clone() },
            list_tag_relations_keyset: ListTagRelationsKeysetUseCase { repo: tags.clone() },
            update_tags: UpdateTagsUseCase { repo: tags.clone() },
            update_tag_relations: UpdateTagRelationsUseCase { repo: tags },
            //  Files
            get_file: GetFileUseCase { repo: files },
        }
    }
}
