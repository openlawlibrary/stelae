//! The http module helper functions useful for serving http content
use mime_guess::from_ext as file_extension_to_mime;

use actix_web::http::header::ContentType;
use std::path::Path;

/// `get_contenttype` uses the file extension to return the `ContentType`
/// for the content at `path`.
///
/// If there is no extension, we assume it is
/// html. If the extension cannot be converted to a str, then we return
/// HTML.
/// Some browsers will not render `application/rdf+xml`, but instead will
/// download it. So we instead return `text/plain` for `.rdf` files.
#[must_use]
pub fn get_contenttype(path: &str) -> ContentType {
    let extension = Path::new(&path)
        .extension()
        .map_or("html", |ext| ext.to_str().map_or("", |ext_str| ext_str));
    let mime = file_extension_to_mime(extension).first_or(mime::TEXT_HTML);
    if (mime.type_(), mime.subtype().as_str()) == (mime::APPLICATION, "rdf") {
        return ContentType(mime::TEXT_PLAIN);
    }
    ContentType(mime)
}

#[cfg(test)]
mod test {
    use crate::utils::http::get_contenttype;

    #[test]
    fn test_get_contenttype_when_html_ext_expect_html() {
        let cut = get_contenttype;
        let actual = cut("a/b.html").to_string();
        let expected = String::from("text/html");
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_get_contenttype_when_no_ext_expect_html() {
        let cut = get_contenttype;
        let actual = cut("a/b").to_string();
        let expected = String::from("text/html");
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_get_contenttype_when_png_ext_expect_image() {
        let cut = get_contenttype;
        let actual = cut("a/b.png").to_string();
        let expected = String::from("image/png");
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_get_contenttype_when_xml_ext_expect_xml() {
        let cut = get_contenttype;
        let actual = cut("a/b.xml").to_string();
        let expected = String::from("text/xml");
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_get_contenttype_when_rdf_ext_expect_rdf() {
        let cut = get_contenttype;
        let actual = cut("a/b.rdf").to_string();
        let expected = String::from("text/plain");
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_get_contenttype_when_incorrect_ext_expect_html() {
        let cut = get_contenttype;
        let actual = cut("a/b.01").to_string();
        let expected = String::from("text/html");
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_get_contenttype_when_exe_ext_expect_octet_stream() {
        let cut = get_contenttype;
        let actual = cut("a/b.exe").to_string();
        let expected = String::from("application/octet-stream");
        assert_eq!(expected, actual);
    }
}
