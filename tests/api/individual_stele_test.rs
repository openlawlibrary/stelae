use crate::common;
use actix_web::test;

#[actix_web::test]
async fn test_something() {
    common::initialize_archive("individual-stele.sh");
}
