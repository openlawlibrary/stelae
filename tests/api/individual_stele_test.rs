use crate::common;
use crate::common::ArchiveType;
use actix_web::test;

#[actix_web::test]
async fn test_something() {
    common::initialize_archive(ArchiveType::Basic);
}
